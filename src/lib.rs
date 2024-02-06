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
pub use crate::search::LinkageSearch;
pub use crate::search::MemoryAccessSearch;
pub use crate::search::StochasticSearch;

use rand::Rng;
use std::convert::TryInto;

/// Trait for testing a code sequence
pub trait Test<I: Instruction> {
    /// Run the program, and return a score of how well it did
    fn run(&self, program: &Candidate<I>) -> f64;
}

/// Type used to feed back to the SearchAlgorithms. The search algorithms are made to react to this
/// to cull the search space by disallowing certain instructions, and to make sure that generated
/// programs pass static analysis passes.
#[derive(Debug)]
pub enum SearchCull<I: Instruction> {
    /// This instruction is okay, no need to cull the search space.
    Okay,

    /// The instruction is filtered away, and in so doing we want to suggest an instruction that
    /// wouldn't be filtered away. So this is like calling `instruction.increment()` until the
    /// condition to filter for no longer holds. In this way, the exhaustive search can cull huge
    /// swathes of of the search space by static analysis.
    SkipTo(Option<I>),
}

impl<I: Instruction> SearchCull<I> {
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

pub trait Instruction: Copy + Clone + std::marker::Send + std::fmt::Display {
    //! A trait for any kind of machine instruction. The searches use this trait to mutate
    //! candidate programs, the emulators use this trait to get at a byte stream encoding a
    //! candidate program.

    /// Return a random machine instruction
    fn random() -> Self;

    /// Mutates a machine instruction. This consumes self, and returns another machine instruction
    /// which is similar, but may be different. The difference will of course depend on the
    /// targetted machine; but differences could include a changed operand, or swapping an
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
#[derive(Clone, Debug, Default)]
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
pub trait Compatibility<I: Instruction> {
    /// Returns a `SearchCull` to either say the instruction is okay, or to say it's not okay and
    /// perhaps get a suggestion to the search algorithm.
    fn check(&self, instruction: &I) -> SearchCull<I>;
}

/// Selects instructions to ensure the program begins and ends with the correct instructions.
pub trait Linkage<S: SearchAlgorithm, I: Instruction> {
    /// Returns a `Vec<SearchCull<I>>` to list the problems with the given `Candidate<I>`.
    fn check(&self, search: &mut S, instruction: &Candidate<I>) -> bool;
}

pub trait SearchAlgorithm {
    //! You can use this to guide the search algorithm.

    /// Which instruction set to use
    type Item: Instruction;

    /// Tell the search algorithm about how close it's getting
    fn score(&mut self, score: f32);

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

    /// Adorns the search algorith with a static analysis pass which ensures the program does not
    /// access memory outside of the specified regions
    fn memory_access(self, regions: Vec<core::ops::Range<u16>>) -> MemoryAccessSearch<Self, <Self as SearchAlgorithm>::Item>
        where Self: Sized
    {
        MemoryAccessSearch::new(self, regions)
    }

    /// Returns a `SearchAlgorithmIterator`, which can be used to iterate over the generated
    /// candidates.
    fn iter(&mut self) -> SearchAlgorithmIterator<'_, Self> {
        SearchAlgorithmIterator { inner: self }
    }
}

pub trait HammingDistance<T> {
    //! Trait for calculating the hamming distance of two values, even if they have different
    //! widths.

    /// Returns the values' hamming distance. This is a commutative operations, so
    /// `x.hamming_distance(y)` is equivalent to `y.hamming_distance(x)`.
    fn hamming_distance(self, other: T) -> f32;
}
