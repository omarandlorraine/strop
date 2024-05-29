use crate::z80::Z80Instruction;
use crate::Fixup;
use crate::Instruction;

/// Fixup ensuring that the Z80 instruction is present on the Intel 8080
#[derive(Debug)]
pub struct I8080Compatibility;

impl Fixup<Z80Instruction> for I8080Compatibility {
    fn random(&self, _insn: Z80Instruction) -> Z80Instruction {
        use crate::Instruction;
        loop {
            let insn = Z80Instruction::random();
            if self.check(insn) {
                return insn;
            }
        }
    }

    fn next(&self, insn: Z80Instruction) -> Option<Z80Instruction> {
        insn.increment_opcode()
    }

    fn check(&self, insn: Z80Instruction) -> bool {
        !insn.opcode_present_on_8080()
    }
}

/// Fixup ensuring that the Z80 instruction is present on the SM83 (aka. Gameboy)
#[derive(Debug)]
pub struct Sm83Compatibility;

impl Fixup<Z80Instruction> for Sm83Compatibility {
    fn random(&self, _insn: Z80Instruction) -> Z80Instruction {
        use crate::Instruction;
        loop {
            let insn = Z80Instruction::random();
            if self.check(insn) {
                return insn;
            }
        }
    }

    fn next(&self, insn: Z80Instruction) -> Option<Z80Instruction> {
        match insn.increment_opcode() {
            Some(i) => {
                if self.check(i) {
                    self.next(i)
                } else {
                    Some(i)
                }
            }
            None => None,
        }
    }

    fn check(&self, insn: Z80Instruction) -> bool {
        insn.opcode_present_on_sm83()
    }
}
