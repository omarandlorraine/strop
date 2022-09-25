//! This module defines the `Instruction` trait and pulls in the architecture-
//! specific instruction types.

pub mod mos6502;

/// The `Instruction` trait defines the behaviour of an instruction. It is
/// used by the BasicBlock type, since a basic block is a sequence of
/// instructions.
pub trait Instruction: Clone + Sized {
    /// Length of the instruction, in bytes
    fn length(&self) -> usize;

    /// Returns a random instruction.
    fn new() -> Self
    where
        Self: Sized;

    /// Returns an iterator over the bytes that encode the instruction
    fn as_bytes(&self) -> Box<dyn Iterator<Item = u8>>;
}
