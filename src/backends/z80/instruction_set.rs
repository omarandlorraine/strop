//! A module for the representation of SM83 machine instructions.
use crate::backends::x80::X80;
use crate::backends::x80::data::InstructionData;

/// Represents a Z80 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Instruction([u8; 5]);

impl Instruction {
    fn incr_at_offset(&mut self, offset: usize) {
        for o in (offset + 1)..5 {
            self.0[o] = 0;
        }
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
        while self.decode_inner().is_none() {
            // If there's no instruction data for this instruction it's not a valid prefix/opcode
            // combo, so increment it.
            self.next_opcode().unwrap();
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let data = self.decode();
        write!(f, "{}", data.mnemonic)?;
        let mut first = true;
        let has_displacement  = data.operands.iter().any(|operand| *operand == "(ix+nn)" || *operand == "(iy+nn)");
        for op in data.operands.iter().filter(|op| !op.is_empty()) {
            if first {
                write!(f, " ")?;
                first = false;
            } else {
                write!(f, ", ")?;
            }

            let (displacement, operand, operand2) = if has_displacement {
                (self.0[self.instruction_length() + 0], self.0[self.instruction_length() + 1], self.0[self.instruction_length() + 2])
            } else {
                (self.0[self.instruction_length() + 0], self.0[self.instruction_length() + 0], self.0[self.instruction_length() + 1])
            };

            if *op == "n16" {
                write!(f, "{:x}h", u16::from_le_bytes([operand, operand2]))?;
            } else if *op == "e8" {
                write!(f, "{:x}h", self.0[1])?;
            } else if *op == "sp+e8" {
                write!(f, "sp+{:x}h", self.0[1])?;
            } else if *op == "(a8)" {
                write!(f, "({:x}h)", self.0[1])?;
            } else if *op == "n8" {
                write!(f, "{:x}h", operand)?;
            } else if *op == "a16" {
                write!(f, "{:x}h", u16::from_le_bytes([operand, operand2]))?;
            } else if *op == "(a16)" {
                write!(f, "({:x}h)", u16::from_le_bytes([operand, operand2]))?;
            } else if *op == "(ix+nn)" {
                write!(f, "(ix+{:x}h)", displacement)?;
            } else if *op == "(iy+nn)" {
                write!(f, "(iy+{:x}h)", displacement)?;
            } else {
                write!(f, "{op}")?
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
        Self([random(), random(), random(), random(), random()])
    }
    fn first() -> Self {
        Self([00, 0, 0, 0, 0])
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
            4 => vec![self.0[0], self.0[1], self.0[2], self.0[3]],
            5 => vec![self.0[0], self.0[1], self.0[2], self.0[3], self.0[4]],
            _ => unreachable!(),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut insn = Self::first();

        // first copy in the prefixes if any and opcode
        insn.0[0] = *bytes.get(0)?;
        if matches!(insn.0[0], 0xcb | 0xed | 0xdd | 0xfd) {
            insn.0[1] = *bytes.get(1)?;
        }
        if matches!(insn.0[0], 0xdd | 0xfd) && insn.0[1] == 0xcb {
            insn.0[2] = *bytes.get(2)?;
        }

        // is it a valid prefix/opcode mashup?
        insn.decode_inner()?;

        // now we know the length of the instruction we can copy the operand in
        match insn.to_bytes().len() {
            1 => {}
            2 => {
                insn.0[1] = *bytes.get(1)?;
            }
            3 => {
                insn.0[1] = *bytes.get(1)?;
                insn.0[2] = *bytes.get(2)?;
            }
            4 => {
                insn.0[1] = *bytes.get(1)?;
                insn.0[2] = *bytes.get(2)?;
                insn.0[3] = *bytes.get(3)?;
            }
            5 => {
                insn.0[1] = *bytes.get(1)?;
                insn.0[2] = *bytes.get(2)?;
                insn.0[3] = *bytes.get(3)?;
                insn.0[4] = *bytes.get(4)?;
            }
            _ => unreachable!(),
        }
        Some(insn)
    }
}

impl Instruction {
    fn decode_inner(&self) -> Option<&'static InstructionData> {
        match self.0[0] {
            0xcb => crate::backends::sm83::data::CBPREFIXED[self.0[1] as usize].as_ref(),
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
            _ => {
                let z80 = crate::backends::z80::data::UNPREFIXED[self.0[0] as usize].as_ref();
                if z80.is_some() {
                    return z80;
                }
                crate::backends::i8080::data::UNPREFIXED[self.0[0] as usize].as_ref()
            }
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
        } else {
            self.incr_at_offset(self.instruction_length() - 1);
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
        let cb = if self.0[1] == 0xcb { 1 } else { 0 };
        match self.0[0] {
            0xcb => 2,
            0xed => 2,
            0xdd => 2 + cb,
            0xfd => 2 + cb,
            _ => 1,
        }
    }
}
