//! Superoptimizer written in Rust
//! ------------------------------
//! This program stochastically generates assembly language programs that compute a given function.
//! Strop provides mechanisms for generating programs, and mutating them in ways that
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

#[cfg(feature = "z80")]
pub mod z80;

mod scalar;
pub mod search;

pub use crate::search::BruteForceSearch;
pub use crate::search::SearchTrace;
pub use crate::search::StochasticSearch;

use rand::Rng;
use std::convert::TryInto;

/// An object implementing this trait is a static analysis on the instruction level. Usefully culls
/// the search space by eliminating instructions not present on a particular model, or instructions
/// accessing memory outside of permissible ranges, or any instruction that's not a "return from
/// subroutine" instruction, or ...
pub trait Fixup<I: Instruction>: std::fmt::Debug {
    /// Fixes an instruction up by randomly selecting a different instruction
    fn random(&self, insn: I) -> I;
    /// Fixes an instruction up by iterating to the next instruction
    fn next(&self, insn: I) -> Option<I>;
    /// Checks whether this fixup needs to alter this instruction
    fn check(&self, insn: I) -> bool;
}

/// A fixup (see the Fixup trait) which yields exactly one instruction. Useful for ensuring that,
/// for example, an interrupt handler ends in that architecture's "Return From Interrupt"
/// instruction.
#[derive(Debug)]
pub struct SingleInstruction<I: Instruction>(I);

impl<I: Instruction> crate::Fixup<I> for SingleInstruction<I> {
    fn check(&self, insn: I) -> bool {
        insn != self.0
    }

    fn next(&self, insn: I) -> Option<I> {
        if insn < self.0 {
            Some(self.0)
        } else {
            None
        }
    }

    fn random(&self, _insn: I) -> I {
        self.0
    }
}

/// A fixup (see the Fixup trait) which calls any number of fixups.
pub struct FixupGroup<I: Instruction>(Vec<Box<dyn Fixup<I>>>);

impl<I: Instruction> std::fmt::Debug for FixupGroup<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        write!(
            f,
            "FixupGroup {{{}}}",
            self.0.iter().fold(String::new(), |mut output, f| {
                let _ = write!(output, "{:?}, ", f);
                output
            })
        )
    }
}

impl<I: Instruction + std::fmt::Debug> crate::Fixup<I> for FixupGroup<I> {
    fn check(&self, insn: I) -> bool {
        self.0.iter().any(|f| f.check(insn))
    }

    fn random(&self, insn: I) -> I {
        let mut changed = false;
        let mut insn = insn;
        for f in &self.0 {
            if f.check(insn) {
                changed = true;
                insn = f.random(insn);
            }
        }
        if changed {
            self.random(insn)
        } else {
            insn
        }
    }

    fn next(&self, insn: I) -> Option<I> {
        let mut changed = false;
        let mut insn = insn;
        for f in &self.0 {
            if f.check(insn) {
                changed = true;
                insn = f.next(insn)?;
            }
        }
        if changed {
            self.next(insn)
        } else {
            Some(insn)
        }
    }
}

pub trait Instruction:
    std::cmp::PartialOrd
    + Copy
    + Clone
    + std::marker::Send
    + std::fmt::Display
    + Sized
    + std::fmt::Debug
    + Default
{
    //! A trait for any kind of machine instruction. The searches use this trait to mutate
    //! candidate programs, the emulators use this trait to get at a byte stream encoding a
    //! candidate program.

    /// Return a random machine instruction
    fn random() -> Self;

    /// Mutates a machine instruction in place. The mutation will of course depend on the
    /// targeted machine; but differences could include a changed operand, or swapping an
    /// increment for a decrement, etc.
    fn mutate(&mut self);

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
    pub fn disassemble(&self, label: &str) {
        println!("{}:", label);
        for insn in &self.instructions {
            println!("\t{}", insn);
        }
    }

    /// Returns the number of instructions in the candidate program
    pub fn length(&self) -> usize {
        self.instructions.len()
    }

    /// replaces an instruction at a given offset
    pub fn replace(&mut self, offset: usize, instruction: T) {
        self.instructions[offset] = instruction;
    }

    /// offset of the last instruction
    pub fn last_offset(&self) -> usize {
        self.instructions.len() - 1
    }

    /// replaces the last instruction
    pub fn replace_last(&mut self, instruction: T) {
        self.replace(self.last_offset(), instruction)
    }
}

/// An adapter struct for iterating over the candidate programs generated by a `SearchAlgorithm`.
#[derive(Debug)]
pub struct SearchAlgorithmIterator<'a, T: SearchAlgorithm + ?Sized>(&'a mut T);

impl<'a, T> Iterator for SearchAlgorithmIterator<'a, T>
where
    T: SearchAlgorithm,
{
    type Item = Candidate<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.generate()
    }
}

pub trait SearchAlgorithm: Clone {
    //! You can use this to guide the search algorithm.

    /// Which instruction set to use
    type Item: Instruction;

    /// Tell the search algorithm about how close it's getting
    fn score(&mut self, score: f32);

    /// Tell the search algorithm that an instruction is incorrectly placed; the `fixup` object may
    /// be queried for correct instructions at this offset. Returns `true` if the instruction was
    /// replaced, and false otherwise.
    fn replace<F: Fixup<Self::Item>>(&mut self, offset: usize, fixup: F) -> bool;

    /// Get the next Candidate
    fn generate(&mut self) -> Option<Candidate<Self::Item>>;

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

    /// Returns a reference to the Candidate which will be generated next
    fn peek(&self) -> &Candidate<Self::Item>;

    /// Starts or restarts the search from the given point in the search space.
    fn start_from(&mut self, point: Candidate<Self::Item>);

    /// Returns a `SearchAlgorithmIterator`, which can be used to iterate over the generated
    /// candidates.
    fn iter(&mut self) -> SearchAlgorithmIterator<'_, Self> {
        SearchAlgorithmIterator(self)
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
