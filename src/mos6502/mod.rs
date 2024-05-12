//! Module containing everything needed to use Strop to generate code for the MOS 6502

pub mod emulators;
pub mod instruction_set;

use crate::mos6502::instruction_set::Cmos6502Instruction;
use crate::Fixup;

/// A fixup ensuring compatibility with the very early chips, before the ROR instruction was
/// implemented.
#[derive(Debug)]
pub struct RevisionA;

/// A fixup ensuring that the instruction does not trigger the Indirect JMP bug that's present on
/// NMOS chips.
#[derive(Debug)]
pub struct IndirectJmp;

/// A fixup ensuring compatibility with a wide range of 6502's. This means not exercizing any of
/// the CMOS extensions or decimal mode.
#[derive(Debug)]
pub struct SafeBet;

impl Fixup<Cmos6502Instruction> for IndirectJmp {
    fn check(&self, insn: Cmos6502Instruction) -> bool{
        use crate::Instruction;
        let enc = insn.encode();
        (enc[0], enc[1]) != (0x6c, 0xff)
    }

    fn random(&self, insn: Cmos6502Instruction) -> Cmos6502Instruction {
        instruction_set::randomize_operand(insn)
    }

    fn next(&self, insn: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
        instruction_set::increment_operand(insn)
    }
}

impl Fixup<Cmos6502Instruction> for RevisionA {
    fn check(&self, insn: Cmos6502Instruction) -> bool {
        use crate::Instruction;
        matches!(insn.encode()[0], 0x66 | 0x6a | 0x6e | 0x76 | 0x7e)
    }

    fn random(&self, _insn: Cmos6502Instruction) -> Cmos6502Instruction {
        use crate::Instruction;
        Cmos6502Instruction::random()
    }

    fn next(&self, insn: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
        instruction_set::increment_operand(insn)
    }
}
