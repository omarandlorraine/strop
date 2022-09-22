//! This module defines the `Instruction` trait and pulls in the architecture-
//! specific modules.

pub mod mos6502;

/// trait for things which may be mutated. This should include `Instruction`,
/// `BasicBlock`, etc
pub trait Strop {
    /// Does a random mutation to the object
    fn mutate(&mut self);

    /// creates a new object, having a random value
    fn random() -> Self
    where
        Self: Sized;
}

/// The `Instruction` trait defines the behaviour of an instruction. It is
/// used by the BasicBlock type, since a basic block is a sequence of
/// instructions.
pub trait Instruction: std::fmt::Display + Clone + Sized {
    /// The state on which the instruction can operate
    type State: Default;

    /// Randomly mutates the instruction
    fn randomize(&mut self);

    /// The length of the instruction in machine words
    fn length(&self) -> usize;

    /// Execute the instruction
    fn operate(&self, s: &mut Self::State);

    /// Returns a random instruction.
    fn new() -> Self
    where
        Self: Sized;

    /// Returns an iterator over the bytes that encode the instruction
    fn as_bytes(&self) -> Box<dyn Iterator<Item = u8>>;
}
