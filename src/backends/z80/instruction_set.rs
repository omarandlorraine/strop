//! A module for the representation of SM83 machine instructions.
use crate::backends::x80::X80;
use crate::backends::x80::data::InstructionData;

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Instruction([u8; 3]);

impl Instruction {
    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
        while crate::backends::z80::data::UNPREFIXED[self.0[0] as usize].is_none()
            && self.0[0] != 0xcb
            && self.0[0] != 0xed
            && self.0[0] != 0xdd
            && self.0[0] != 0xfd
        {
            // If there's no instruction data for this instruction (or more precisely, the
            // first byte of the instruction) is invalid. this is because there are no invalid
            // but prefixed instructions
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
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut insn = Self::first();
        insn.0[0] = bytes[0];
        match insn.to_bytes().len() {
            1 => {}
            2 => {
                insn.0[1] = bytes[1];
            }
            3 => {
                insn.0[1] = bytes[1];
                insn.0[2] = bytes[2];
            }
            _ => unreachable!(),
        }
        insn
    }
}

impl Instruction {
    fn decode_inner(&self) -> Option<&'static InstructionData> {
        match self.0[0] {
            0xcb => crate::backends::z80::data::CBPREFIXED[self.0[1] as usize].as_ref(),
            0xed => crate::backends::z80::data::EDPREFIXED[self.0[1] as usize].as_ref(),
            0xdd => {
                if self.0[1] == 0xcb {
                    crate::backends::z80::data::DDCBPREFIXED[self.0[1] as usize].as_ref()
                } else {
                    crate::backends::z80::data::DDPREFIXED[self.0[1] as usize].as_ref()
                }
            }
            0xfd => {
                if self.0[1] == 0xcb {
                    crate::backends::z80::data::FDCBPREFIXED[self.0[1] as usize].as_ref()
                } else {
                    crate::backends::z80::data::FDPREFIXED[self.0[1] as usize].as_ref()
                }
            }
            _ => crate::backends::z80::data::UNPREFIXED[self.0[0] as usize].as_ref(),
        }
    }
}

impl X80 for Instruction {
    type Emulator = crate::backends::z80::emu::Emulator;

    fn decode(&self) -> &'static InstructionData {
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
        match self.0[0] {
            0xcb => 2,
            0xed => 2,
            0xdd => {
                if self.0[1] == 0xcb {
                    3
                } else {
                    2
                }
            }
            0xfd => {
                if self.0[1] == 0xcb {
                    3
                } else {
                    2
                }
            }
            _ => 1,
        }
    }
}
