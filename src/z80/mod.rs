//! The Z80 backend (can of course also be used to generate code for the Intel 8080 or the SM83).

pub mod emulators;
pub mod instruction_set;

/// Returns the default Z80 instruction set
pub fn z80() -> instruction_set::Z80InstructionSet {
    instruction_set::Z80InstructionSet::default()
}
