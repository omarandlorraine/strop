//! A module for the representation of Z80 machine instructions.

/// Represents a Z80 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 5]);

impl crate::Step for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn next(&mut self) -> crate::IterationResult {
        use crate::Encode;
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup()
        }
    }
}

impl Insn {
    /// constructs a return instruction `ret`
    pub fn ret() -> Self {
        Self::new(&[0xc9])
    }

    /// Returns `true` if the instruction does any kind of flow control, `false` otherwise
    pub fn is_flow_control(&self) -> bool {
        match self.0[0] {
            0x10 => /*djnz*/ true,
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => /* jr */ true,
            0x76 => /* halt */ true,
            0xc0 | 0xc8 | 0xc9 | 0xd0 | 0xd8 | 0xe0 | 0xe8 | 0xf0 | 0xf8 /* ret */ => true,
            0xc2 | 0xd2 | 0xe2 | 0xf2 | 0xc3 | 0xca | 0xda | 0xea | 0xfa | 0xe9 /* jp */ => true,
            0xed => /*reti*/ self.0[1] == 0x4d,
            0xdd => /* jp */ self.0[1] == 0xe9,
            0xfd => /* jp */ self.0[1] == 0xe9,
            _ => false,
        }
    }

    /// Returns true if the instruction is permitted in pure functions
    pub fn allowed_in_pure_functions(&self) -> bool {
        if self.0[0] == 0xcb {
            // it uses (hl)
            return self.0[1] & 0x07 != 0x06;
        }
        !matches!(
            self.0[0],
            0x02 | 0x0a
                | 0x12
                | 0x22
                | 0x2a
                | 0x32
                | 0x34
                | 0x35
                | 0x36
                | 0x3a
                | 0x77
                | 0xd3
                | 0xdb
        )
    }

    /// Increments the opcode, and sets all subsequent bytes (i.e. the operand) to 0.
    pub fn next_opcode(&mut self) -> crate::IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else if self.0[0] == 0xcb && self.0[1] < 0xff {
            self.0[1] += 1;
            self.0[2] = 0;
            self.0[3] = 0;
            self.0[4] = 0;
            self.fixup()
        } else {
            self.0[0] += 1;
            self.0[1] = 0;
            self.0[2] = 0;
            self.0[3] = 0;
            self.0[4] = 0;
            self.fixup()
        }
    }
}

impl crate::subroutine::ShouldReturn for Insn {
    fn should_return(&self, offset: usize) -> Result<(), crate::StaticAnalysis<Self>> {
        if self.0[0] == 0xc9 {
            return Ok(());
        }
        Err(crate::StaticAnalysis::<Self> {
            offset,
            advance: Self::next_opcode,
            reason: "ShouldReturn",
        })
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.decode().to_bytes()
    }

    fn len(&self) -> usize {
        self.decode().to_bytes().len()
    }
}

impl crate::Mutate for Insn {
    fn random() -> Self {
        let mut s = Self(rand::random());
        s.fixup().ok(); // TODO, check for this condition and put it right
        s
    }

    fn mutate(&mut self) {
        use rand::Rng;
        let offset = rand::rng().random_range(0..5);

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u8 = 1 << rand::rng().random_range(0..8);
            self.0[offset] ^= mask;
        } else {
            // could completely change the instruction word to something completely different
            self.0[offset] = rand::random()
        }

        self.fixup().ok(); // TODO, check for this condition and put it right
    }
}

impl crate::Branch for Insn {}

impl Insn {
    /// Constructs a new Insn from a slice of bytes
    pub fn new(mc: &[u8]) -> Self {
        let mut enc = [0, 0, 0, 0, 0];
        enc[..mc.len().min(5)].copy_from_slice(mc);
        Self(enc)
    }

    /// Decodes the instruction and returns a `dez80::Instruction`.
    pub fn decode(&self) -> dez80::Instruction {
        let encoding = Vec::<_>::from(self.0);
        let e = dez80::Instruction::decode_one(&mut encoding.as_slice());
        match e {
            Ok(e) => e,
            Err(e) => panic!(
                "couldn't encode {:?}: {:?}",
                self.0
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<String>>()
                    .join(" "),
                e
            ),
        }
    }

    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
    }

    fn fixup(&mut self) -> crate::IterationResult {
        if matches!(self.0[0], 0xdd | 0xed) {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [
                0x44, 0x4c, 0x54, 0x5c, 0x60, 0x77, 0x7c, 0x84, 0x8c, 0x94, 0x9c, 0xa4, 0xac, 0xb4,
                0xbc, 0xcb, 0xe1, 0xe3, 0xe5, 0xe9, 0xf9,
            ] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return Ok(());
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
            return Ok(());
        }

        if self.0[0] == 0xfd {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [0x09] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return Ok(());
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_opcodes() {
        use super::Insn;
        use crate::Step;

        let mut insn = Insn::first();
        while insn.next().is_ok() {
            assert!(insn.decode().ignored_prefixes.is_empty());
        }
    }
}
