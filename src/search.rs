//! Module defining the Searchable trait

use crate::StaticAnalysis;
use crate::Sequence;
use crate::test::Vals;
use crate::IterationResult;

pub trait Instruction: Clone {
    /// Returns the first instruction
    ///
    /// This is usually the instruction sorting the first numerically, so it has the value 0 or
    /// something.
    fn first() -> Self;

    /// Returns a random instruction
    fn random() -> Self;

    /// Increments the instruction in place
    ///
    /// This includes any operands, prefixes and whatever else
    fn increment(&mut self) -> IterationResult;

    /// Mutates the instruction in place
    ///
    /// This includes any operands, prefixes and whatever else
    fn mutate(&mut self);
}

/// The Searchable trait. This trait is implemented for functions which also are machine code
/// sequences, so that clients can perform exhaustive or stochastic searches.
pub trait Searchable<P: Vals, R: Vals> {

    /// The type of instruction making up this searchable function
    type Instruction: crate::search::Instruction;

    /// Performs static analysis on the instruction sequence, possibly yielding a Fixup
    fn analyse(&self) -> StaticAnalysis<Self::Instruction>;

    /// Apply all Fixups yielded by static analysis.
    fn fixup(&mut self) -> IterationResult;

    /// Returns the inner `Sequence<Insn>`, the sequence of instructions.
    fn as_sequence(&self) -> &Sequence<Self::Instruction>;

    /// Returns the inner `Sequence<Insn>`, the sequence of instructions.
    fn as_sequence_mut(&mut self) -> &mut Sequence<Self::Instruction>;

    /// Take one step through the search space
    fn step(&mut self) -> IterationResult {
        self.as_sequence_mut().next()?;
        self.fixup()
    }
}
