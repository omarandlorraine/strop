//! A module for the representation of SM83 machine instructions.
use crate::static_analysis::Fixup;
use crate::x80::data::InstructionData;
use crate::{IterationResult, StepError};

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 3]);

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} ", self.decode().as_ref().unwrap().mnemonic)?;
        for i in self
            .decode()
            .as_ref()
            .unwrap()
            .operands
            .iter()
            .filter(|i| !i.is_empty())
        {
            write!(f, "{i} ")?;
        }
        Ok(())
    }
}

impl crate::Disassemble for Insn {
    fn dasm(&self) {
        let mut operands = String::new();
        let data = self.decode().as_ref().unwrap();
        for op in data.operands.iter().filter(|op| !op.is_empty()) {
            if !operands.is_empty() {
                operands.push_str(", ");
            }
            if *op == "n16" {
                operands.push_str(&format!(
                    "{:x}h",
                    u16::from_le_bytes([self.0[1], self.0[2]])
                ));
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

impl crate::x80::X80 for Insn {
    type Emulator = crate::sm83::emu::Emu;
}

impl crate::ShouldReturn for Insn {
    fn should_return(&self, offset: usize) -> crate::StaticAnalysis<Self> {
        Fixup::check(self.0[0] == 0xc9, "ShouldReturn", Self::next_opcode, offset)
    }
}

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self}     \t;")?;
        for i in 0..self.decode().as_ref().unwrap().bytes {
            write!(f, "{:#02x}", self.0[i])?;
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
            while self.decode().is_none() {
                // If there's no instruction data for this instruction then the precisely, the
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
        self.decode().as_ref().map(|data| data.bytes).unwrap()
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

    fn decode(&self) -> &'static Option<InstructionData> {
        if self.0[0] == 0xcb {
            return &crate::sm83::data::CBPREFIXED[self.0[1] as usize];
        }
        &crate::sm83::data::UNPREFIXED[self.0[0] as usize]
    }

    fn next_opcode(&mut self) -> IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else {
            self.0[0] += 1;
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Insn;
    use crate::Step;

    fn instruction_asserts(i: &Insn) {
        let arr = i.0;
        assert!(i.decode().is_some(), "check_instruction(Insn({arr:?}))");
        format!("{i}");
        format!("{i:?}");
    }

    #[test]
    fn all() {
        let mut i = Insn::first();
        while i.next().is_ok() {
            instruction_asserts(&i);
        }
    }
}
