//! A module for the representation of SM83 machine instructions.
use crate::backends::x80::X80;
use crate::backends::x80::data::InstructionData;

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Instruction([u8; 3]);

impl Instruction {
    fn decode_inner(&self) -> Option<InstructionData> {
        use crate::Instruction as _;

        if matches!(self.0[0], 0xe3 | 0xeb | 0xec) {
            // opcodes in i8080 which are removed in sm83
            return None;
        }
        if self.0[0] == 0xcb {
            return crate::backends::sm83::data::CBPREFIXED[self.0[1] as usize].clone();
        }
        let sm83 = crate::backends::sm83::data::UNPREFIXED[self.0[0] as usize].clone();

        if sm83.is_some() {
            return sm83;
        }
        Some(crate::backends::i8080::Instruction::from_bytes(&self.to_bytes())?.decode())
    }

    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
        while self.decode_inner().is_none() && self.0[0] != 0xcb {
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
        write!(f, "{}", data.mnemonic)?;
        let mut first = true;
        for op in data.operands.iter().filter(|op| !op.is_empty()) {
            if first {
                write!(f, " ")?;
                first = false;
            } else {
                write!(f, ", ")?;
            }

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
        let mut insn = Self::first();
        insn.0[0] = *bytes.first()?;
        match insn.to_bytes().len() {
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

impl X80 for Instruction {
    type Emulator = crate::backends::sm83::emu::Emu;

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

        crate::Fixup::<Self>::check(
            self.0[0] == INSN,
            "DoesNotReturn",
            |i| {
                if i.0[0] <= INSN {
                    i.0[0] = INSN;
                    Ok(())
                } else {
                    Err(crate::StepError::End)
                }
            },
            0,
        )
    }

    fn instruction_length(&self) -> usize {
        if self.0[0] == 0xcb { 2 } else { 1 }
    }
}
