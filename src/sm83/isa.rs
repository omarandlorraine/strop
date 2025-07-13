//! A module for the representation of SM83 machine instructions.
use crate::static_analysis::Fixup;
use crate::x80::X80;
use crate::x80::data::InstructionData;
use crate::{IterationResult, StepError};

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 3]);

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} ", self.decode().mnemonic)?;
        for i in self.decode().operands.iter().filter(|i| !i.is_empty()) {
            write!(f, "{i} ")?;
        }
        Ok(())
    }
}

impl crate::Disassemble for Insn {
    fn dasm(&self) {
        let mut operands = String::new();
        let data = self.decode();
        for op in data.operands.iter().filter(|op| !op.is_empty()) {
            if !operands.is_empty() {
                operands.push_str(", ");
            }
            if *op == "n16" {
                operands.push_str(&format!(
                    "{:x}h",
                    u16::from_le_bytes([self.0[1], self.0[2]])
                ));
            } else if *op == "n8" {
                operands.push_str(&format!("{:x}h", self.0[1]));
            } else if *op == "(a16)" {
                operands.push_str(&format!(
                    "({:x}h)",
                    u16::from_le_bytes([self.0[1], self.0[2]])
                ));
            } else {
                operands.push_str(op);
            }
        }

        println!("\t{} {}", data.mnemonic, operands);
    }
}

impl X80 for Insn {
    type Emulator = crate::sm83::emu::Emu;

    fn decode(&self) -> &'static InstructionData {
        if self.0[0] == 0xcb {
            return crate::sm83::data::CBPREFIXED[self.0[1] as usize]
                .as_ref()
                .unwrap();
        }
        crate::sm83::data::UNPREFIXED[self.0[0] as usize]
            .as_ref()
            .unwrap_or_else(|| panic!("no such opcode for {:02x}", self.0[0]))
    }

    fn next_opcode(&mut self) -> IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else if self.0[0] == 0xcb {
            if self.0[1] == 0xff {
                self.0[0] += 1;
                self.0[1] = 0;
            } else {
                self.0[1] += 1;
            }
            self.0[2] = 0;
            Ok(())
        } else {
            self.0[0] += 1;
            while crate::sm83::data::UNPREFIXED[self.0[0] as usize].is_none() {
                self.0[0] += 1;
            }
            Ok(())
        }
    }
}

impl crate::ShouldReturn for Insn {
    fn should_return(&self, offset: usize) -> crate::StaticAnalysis<Self> {
        Fixup::check(self.0[0] == 0xc9, "ShouldReturn", Self::next_opcode, offset)
    }
}

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self}     \t;")?;
        for i in 0..self.decode().bytes {
            write!(f, " {:02x}", self.0[i])?;
        }
        Ok(())
    }
}

impl crate::Step for Insn {
    fn first() -> Self {
        Self([0, 0, 0])
    }

    fn next(&mut self) -> IterationResult {
        use crate::Encode;
        let len = self.len();
        if self.0[0] == 0xff {
            Err(StepError::End)
        } else {
            self.incr_at_offset(len - 1);
            while crate::sm83::data::UNPREFIXED[self.0[0] as usize].is_none() && self.0[0] != 0xcb {
                // If there's no instruction data for this instruction (or more precisely, the
                // first byte of the instruction) is invalid. this is because there are no invalid
                // but prefixed instructions
                self.0[0] += 1;
                self.0[1] = 0;
                self.0[2] = 0;
            }
            Ok(())
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn len(&self) -> usize {
        self.decode().bytes
    }

    fn encode(&self) -> Vec<u8> {
        let mut encoding = self.0.to_vec();
        encoding.truncate(self.len());
        encoding
    }
}

impl Insn {
    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Insn;
    use crate::Step;

    fn instruction_asserts(i: &Insn) {
        use crate::x80::X80;
        use crate::x80::data::ReadWrite;
        // these method calls can panic, let's make sure they don't
        let data = i.decode();
        let _ = format!("{i}");
        let _ = format!("{i:?}");

        if data.operands.contains(&"(hl)") {
            if !data.operands.contains(&"l") {
                assert_eq!(data.h, ReadWrite::R, "{i:?} doesn't read h");
                assert_eq!(data.l, ReadWrite::R, "{i:?} doesn't read l");
            }
        }

        if !["jr", "jp", "call", "ret"].contains(&data.mnemonic) {
            if data.operands.contains(&"c") {
                assert_ne!(data.c, ReadWrite::N, "{i:?} doesn't touch c");
            }
        }

        if ["sbc", "sub", "adc", "add", "and", "xor", "or"].contains(&data.mnemonic)
            && data.operands[0] == "a"
        {
            assert_eq!(data.a, ReadWrite::Rmw, "{i:?} doesn't rmw a");
        }

        if ["res", "set"].contains(&data.mnemonic) {
            if !data.operands.contains(&"(hl)") {
                assert_eq!(
                    [data.a, data.b, data.c, data.d, data.e, data.h, data.l]
                        .iter()
                        .filter(|d| **d == ReadWrite::Rmw)
                        .count(),
                    1,
                    "{i:?} doesn't rmw anything"
                );
            }
        }

        if ["cp"].contains(&data.mnemonic) {
            assert_eq!(data.a, ReadWrite::R, "{i:?} doesn't read a");
        }

        if data.mnemonic == "push" {
            assert!(
                [data.a, data.b, data.c, data.d, data.e, data.h, data.l]
                    .iter()
                    .find(|d| d.writes())
                    .is_none(),
                "{i:?} writes to its operand"
            );
        }
        // check "push" doesn't write to any regs
    }

    #[test]
    fn all() {
        let mut i = Insn::first();
        while i.next().is_ok() {
            instruction_asserts(&i);
        }
    }
}
