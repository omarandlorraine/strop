//! A module for the representation of Z80 machine instructions.

/// Represents a Z80 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 5]);

impl crate::Iterable for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn step(&mut self) -> bool {
        use crate::Encode;
        if self.0[0] == 0xff {
            false
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup();
            true
        }
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

    fn fixup(&mut self) {
        if matches!(self.0[0], 0xdd | 0xed) {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [
                0x44, 0x4c, 0x54, 0x5c, 0x60, 0x77, 0x7c, 0x84, 0x8c, 0x94, 0x9c, 0xa4, 0xac, 0xb4,
                0xbc, 0xcb, 0xe1, 0xe3, 0xe5, 0xe9, 0xf9,
            ] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return;
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
            return;
        }

        if self.0[0] == 0xfd {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [0x09] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return;
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_opcodes() {
        use super::Insn;
        use crate::Encode;
        use crate::Iterable;

        let mut insn = Insn::first();
        assert_eq!(insn.len(), 1);
        while insn.step() {
            println!("{}; {:?}", insn, insn);
            let d = insn.decode();

            if !d.ignored_prefixes.is_empty() {
                let prev = insn;
                while !insn.decode().ignored_prefixes.is_empty() {
                    assert!(!insn.step());
                }
                panic!(
                    "{:?} ({}) has ignored prefixes, next one that doesn't is {:?} ({})",
                    prev, prev, insn, insn
                );
            }
        }
    }
}
