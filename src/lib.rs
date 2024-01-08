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
pub use crate::search::StochasticSearch;

use rand::Rng;
use std::convert::TryInto;

/// Trait for testing a code sequence
pub trait Test<I: Instruction> {
    /// Run the program, and return a score of how well it did
    fn run(&self, program: &Candidate<I>) -> f64;
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

pub trait SearchAlgorithm {
    //! You can use this to guide the search algorithm.

    type Item: Instruction;

    /// Tell the search algorithm about how close it's getting
    fn score(&mut self, score: f32);

    /// Tell the search algorithm that an instruction is incorrect; also propose a correction (this
    /// is to make sure that all proposed programs pass static analysis, for example)
    fn replace(&mut self, offset: usize, instruction: Self::Item);

    /// Get the next Candidate
    fn generate(&mut self) -> Option<Candidate<Self::Item>>;

    fn iter(&mut self) -> SearchAlgorithmIterator<Self> {
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
