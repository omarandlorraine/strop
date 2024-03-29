//! Superoptimizer written in Rust
//! ------------------------------
//! This program stochastically generates assembly language programs that compute a given function.
//! Strop provides mechanisms for generating programs, and mutating them is ways that
//! stochastically approach the desired output.
//!
//! Another way to describe strop, is that it randomly generates pretty good assembly programs.
//!
//! So far, strop has had a focus on supporting architectures that are not well supported by
//! mainstream compilers such as LLVM. These have included architectures common in low-end
//! embedded, and hobbyist retrocomputing.
//!

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "armv4t")]
pub mod armv4t;

#[cfg(feature = "mos6502")]
pub mod mos6502;

#[cfg(feature = "mos6502")]
pub mod robo6502;
pub mod search;

#[cfg(feature = "z80")]
pub mod z80;

mod hamming;

pub use crate::search::BruteForceSearch;
pub use crate::search::CompatibilitySearch;
pub use crate::search::LengthLimitedSearch;
pub use crate::search::LinkageSearch;
pub use crate::search::SearchTrace;
pub use crate::search::StochasticSearch;

use rand::Rng;
use std::convert::TryInto;

/// Trait enabling a stochastic search over instruction sequences
pub trait Stochastic: Instruction {
    /// Builds a [StochasticSearch] object for the given instruction set
    fn stochastic_search() -> StochasticSearch<Self> {
        StochasticSearch::<Self>::new()
    }
}

/// Trait enabling an exhaustive search over instruction sequences
pub trait Bruteforce: Instruction + PartialOrd {
    /// Builds a [StochasticSearch] object for the given instruction set
    fn bruteforce_search() -> BruteForceSearch<Self> {
        BruteForceSearch::<Self>::new()
    }
}

/// Type used to feed back to the SearchAlgorithms. The search algorithms are made to react to this
/// to cull the search space by disallowing certain instructions, and to make sure that generated
/// programs pass static analysis passes.
#[derive(Debug, PartialEq)]
pub enum SearchCull<I: Instruction + PartialEq> {
    /// This instruction is okay, no need to cull the search space.
    Okay,

    /// The instruction is filtered away, and in so doing we want to suggest an instruction that
    /// wouldn't be filtered away. So this is like calling `instruction.increment()` until the
    /// condition to filter for no longer holds. In this way, the exhaustive search can cull huge
    /// swathes of of the search space by static analysis.
    SkipTo(Option<I>),
}

impl<I: Instruction + PartialEq> SearchCull<I> {
    /// Returns true if the SearchCull is okay
    pub fn is_okay(&self) -> bool {
        matches!(self, SearchCull::<I>::Okay)
    }

    /// Returns the suggested instruction if it exists
    pub fn suggestion(&self) -> Option<I> {
        match self {
            SearchCull::<I>::SkipTo(s) => *s,
            _ => None,
        }
    }
}

pub trait Instruction: Copy + Clone + std::marker::Send + std::fmt::Display + Sized {
    //! A trait for any kind of machine instruction. The searches use this trait to mutate
    //! candidate programs, the emulators use this trait to get at a byte stream encoding a
    //! candidate program.

    /// Return a random machine instruction
    fn random() -> Self;

    /// Mutates a machine instruction. This consumes self, and returns another machine instruction
    /// which is similar, but may be different. The difference will of course depend on the
    /// targeted machine; but differences could include a changed operand, or swapping an
    /// increment for a decrement, etc.
    fn mutate(self) -> Self;

    /// Returns the machine instruction's encoding (i.e., what to write into the emulator's memory
    /// in order to execute this instruction)
    fn encode(self) -> Vec<u8>;

    /// Gets the "first" instruction (this could be the first numerically, or by some other
    /// measure. But it should in any case be one which the `increment` method does not revisit).
    fn first() -> Self;

    /// Increments the instruction's encoding by one, and then returns a clone of self.
    fn increment(&mut self) -> Option<Self>;
}

pub trait Emulator<T: Instruction> {
    //! A trait for executing candidate programs, and scoring them

    /// Puts a candidate program into the emulator's address space at the given address, and then
    /// runs the candidate program.
    fn run(&mut self, addr: usize, candidate: &Candidate<T>);
}

/// A candidate program. This is essentially an ordered list of `Instruction`s.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Candidate<T: Instruction> {
    instructions: Vec<T>,
}

impl<T: Instruction> Candidate<T> {
    /// returns the bytes encoding the program
    pub fn encode(&self) -> Vec<u8> {
        self.instructions.iter().flat_map(|i| i.encode()).collect()
    }

    /// creates an empty program.
    pub fn empty() -> Self {
        Self {
            instructions: vec![],
        }
    }

    /// creates a new candidate from a `Vec<T>`.
    pub fn new(instructions: Vec<T>) -> Self {
        Self { instructions }
    }

    /// inserts an instruction at a random offset
    pub fn insert(&mut self, insn: T) {
        // Because the appropriate trait is not implemented for usize, I have to convert to u32 and
        // back.
        let current_length: u32 = self.instructions.len().try_into().unwrap();
        let insertion_offset = rand::thread_rng().gen_range(0..current_length + 1);
        self.instructions.insert(insertion_offset as usize, insn);
    }

    /// Prints the `Candidate` to stdout
    pub fn disassemble(&self) {
        for insn in &self.instructions {
            println!("\t{}", insn);
        }
    }

    /// Returns the number of instructions in the candidate program
    pub fn length(&self) -> usize {
        self.instructions.len()
    }
}

/// An adapter struct for iterating over the candidate programs generated by a `SearchAlgorithm`.
#[derive(Debug)]
pub struct SearchAlgorithmIterator<'a, T: SearchAlgorithm + ?Sized> {
    inner: &'a mut T,
}

impl<'a, T> Iterator for SearchAlgorithmIterator<'a, T>
where
    T: SearchAlgorithm,
{
    type Item = Candidate<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.generate()
    }
}

/// Selects instructions to ensure compatibility with CPU variants.
pub trait Compatibility<I: Instruction + PartialEq> {
    /// Returns a `SearchCull` to either say the instruction is okay, or to say it's not okay and
    /// perhaps get a suggestion to the search algorithm.
    fn check(&self, instruction: &I) -> SearchCull<I>;
}

/// Selects instructions to ensure the program begins and ends with the correct instructions.
pub trait Linkage<S: SearchAlgorithm, I: Instruction> {
    /// Returns `true` iff the candidate fits the linkage
    fn check(&self, instruction: &Candidate<I>) -> bool;

    /// Gives hints to the search algorithm to get the next candidate to fit the linkage
    fn fixup(&self, search: &mut S, instruction: &Candidate<I>) -> bool;
}

pub trait SearchAlgorithm {
    //! You can use this to guide the search algorithm.

    /// Which instruction set to use
    type Item: Instruction;

    /// Tell the search algorithm about how close it's getting
    fn score(&mut self, score: f32);

    /// Test a given candidate for suitability.
    fn fitness(&mut self, candidate: &Candidate<Self::Item>) -> Fitness;

    /// Tell the search algorithm that an instruction is incorrect; also propose a correction (this
    /// is to make sure that all proposed programs pass static analysis, for example)
    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>);

    /// Get the next Candidate
    fn generate(&mut self) -> Option<Candidate<Self::Item>>;

    /// Adorns the search algorithm with a static analysis pass ensuring compatibility with a given
    /// model.
    fn compatibility<C: Compatibility<Self::Item>>(
        self,
        compatibility: C,
    ) -> CompatibilitySearch<Self, <Self as SearchAlgorithm>::Item, C>
    where
        Self: Sized,
        Self::Item: PartialEq,
    {
        CompatibilitySearch::new(self, compatibility)
    }

    /// Adorns the search algorithm with a static analysis pass which ensures the program's
    /// linkage. For example, it could yield only subroutines, or only interrupt handlers, or ...
    fn linkage<L: Linkage<Self, Self::Item>>(
        self,
        linkage: L,
    ) -> LinkageSearch<Self, <Self as SearchAlgorithm>::Item, L>
    where
        Self: Sized,
    {
        LinkageSearch::new(self, linkage)
    }

    /// Calls the supplied function on each generated program
    fn trace(
        self,
        func: fn(&Candidate<Self::Item>),
    ) -> SearchTrace<Self, <Self as SearchAlgorithm>::Item>
    where
        Self: Sized,
    {
        SearchTrace::new(self, func)
    }

    /// Returns a `SearchAlgorithmIterator`, which can be used to iterate over the generated
    /// candidates.
    fn iter(&mut self) -> SearchAlgorithmIterator<'_, Self> {
        SearchAlgorithmIterator { inner: self }
    }
}

pub trait Scalar: num::cast::AsPrimitive<u32> {
    //! Trait for scalar values that may be a function's parameter, or return value, or something.

    /// Returns a random value
    fn random() -> Self;

    /// Converts the value to i32
    fn as_i32(self) -> i32;

    /// Calculates the hamming distance to another value, after truncating both to the same width.
    fn hamming<T: num::cast::AsPrimitive<u32>>(self, other: T) -> u32;
}

/// After finding a valid solution, we might want to optimize it further. To that end, we want to
/// propose a mutation to the valid but suboptimal solution, and see if it still passes static
/// analysis and still computes the function as expected. This enum is for answering that question.
#[derive(Clone, Copy, Debug)]
pub enum Fitness {
    /// The solution can be rejected for failing static analysis
    FailsStaticAnalysis,

    /// The solution passes static analysis and was run in the emulator; it's fitness score is
    /// supplied.
    Passes(f32),
}
