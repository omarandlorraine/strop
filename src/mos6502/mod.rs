//! Module containing everything needed to use Strop to generate code for the MOS 6502
use crate::Candidate;
use crate::Instruction;

pub mod emulators;
pub mod instruction_set;

use instruction_set::Cmos6502;
use instruction_set::Mos6502;

/// Returns the NMOS 6502 instruction set.
pub fn nmos() -> Mos6502 {
    instruction_set::Mos6502::default()
}

/// Returns the CMOS 6502 instruction set.
pub fn cmos() -> Cmos6502 {
    instruction_set::Cmos6502::default()
}
