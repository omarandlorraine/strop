//! A module for the representation of SM83 machine instructions.
use crate::backends::x80::X80;
use crate::backends::x80::data::InstructionData;

/// Represents a 8080 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Instruction([u8; 3]);

impl Instruction {
    fn incr_at_offset(&mut self, offset: usize) {
        use crate::backends::x80::parse::Opcode;

        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
        while Opcode(self.0[0]).parse().is_none() {
            // If there's no instruction data for this instruction then it's invalid
            self.0[0] += 1;
            self.0[1] = 0;
            self.0[2] = 0;
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let data = self.decode();
        write!(f, "{} ", data.mnemonic)?;
        let mut first = false;
        for op in data.operands.iter().filter(|op| !op.is_empty()) {
            if *op == "n16" {
                write!(f, "{:x}h", u16::from_le_bytes([self.0[1], self.0[2]]))?;
            } else if *op == "e8" {
                write!(f, "{:x}h", self.0[1])?;
            } else if *op == "sp+e8" {
                write!(f, "sp+{:x}h", self.0[1])?;
            } else if *op == "(a8)" {
                write!(f, "({:x}h)", self.0[1])?;
            } else if *op == "n8" {
                write!(f, "{:x}h", self.0[1])?;
            } else if *op == "a16" {
                write!(f, "{:x}h", u16::from_le_bytes([self.0[1], self.0[2]]))?;
            } else if *op == "(a16)" {
                write!(f, "({:x}h)", u16::from_le_bytes([self.0[1], self.0[2]]))?;
            } else {
                write!(f, "{op}")?
            }

            if !first {
                write!(f, ", ")?;
                first = true;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let dasm = format!("{self}");
        write!(f, "{dasm:<20} ;")?;
        for byte in &self.0[..self.decode().bytes] {
            write!(f, " {byte:#04x}")?;
        }
        Ok(())
    }
}

impl crate::Instruction for Instruction {
    fn random() -> Self {
        use rand::random;
        Self([random(), random(), random()])
    }
    fn first() -> Self {
        Self([0, 0, 0])
    }
    fn mutate(&mut self) {
        todo!()
    }
    fn increment(&mut self) -> crate::IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else {
            let len = self.decode().bytes;
            self.incr_at_offset(len - 1);
            while self.decode_inner().is_none() {
                self.incr_at_offset(self.instruction_length() - 1);
            }
            Ok(())
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        match self.decode().bytes {
            1 => vec![self.0[0]],
            2 => vec![self.0[0], self.0[1]],
            3 => vec![self.0[0], self.0[1], self.0[2]],
            _ => unreachable!(),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        use crate::backends::x80::parse::Opcode;
        let mut insn = Self::first();
        insn.0[0] = *bytes.first()?;
        match Opcode(insn.0[0]).parse()?.bytes {
            1 => {}
            2 => {
                insn.0[1] = *bytes.get(1)?;
            }
            3 => {
                insn.0[1] = *bytes.get(1)?;
                insn.0[2] = *bytes.get(2)?;
            }
            _ => unreachable!(),
        }
        Some(insn)
    }
}

impl Instruction {
    fn decode_inner(&self) -> Option<InstructionData> {
        use crate::backends::x80::parse::Opcode;
        Opcode(self.0[0]).parse()
    }
}

impl X80 for Instruction {
    type Emulator = crate::backends::i8080::emu::Emulator;

    fn decode(&self) -> InstructionData {
        self.decode_inner().unwrap()
    }

    fn next_opcode(&mut self) -> crate::IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else if self.0[0] == 0xcb {
            self.incr_at_offset(1);
            Ok(())
        } else {
            self.incr_at_offset(0);
            Ok(())
        }
    }

    fn make_return(&self) -> crate::StaticAnalysis<Self> {
        const INSN: u8 = 0xc9;
        if self.0[0] != INSN {
            return Err(crate::Fixup::<Self> {
                advance: |i| {
                    if i.0[0] <= INSN {
                        i.0[0] = INSN;
                        Ok(())
                    } else {
                        Err(crate::StepError::End)
                    }
                },
                offset: 0,
                reason: "DoesNotReturn",
            });
        }
        Ok(())
    }

    fn instruction_length(&self) -> usize {
        1
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn iterall() {
        use super::Instruction;
        use crate::Instruction as _;
        use crate::backends::x80::X80;
        let mut insn = Instruction::first();
        while insn.increment().is_ok() {
            let _ = format!("{insn}");
            let _ = format!("{insn:?}");
            insn.decode();
        }
    }
}
