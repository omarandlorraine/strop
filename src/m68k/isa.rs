use crate::{IterationResult, StaticAnalysis, StepError};

mod data;

/// Represents one 68000 machine code instruction
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Insn(pub(crate) [u16; 5]);

impl Insn {
    /// Increments the first word of the instruction, and resets all the rest to zero
    pub fn next_opcode(&mut self) -> IterationResult {
        if self.0[0] >= data::LAST_LEGAL_OPCODE {
            return Err(StepError::End);
        }
        self.0[0] += 1;
        self.0[1] = 0;
        self.0[2] = 0;
        self.0[3] = 0;
        self.0[4] = 0;
        self.fixup()
    }

    /// Constructs a new Insn from a slice of bytes
    pub fn new(mc: &[u16]) -> Self {
        let mut enc = [0, 0, 0, 0, 0];
        enc[..mc.len().min(5)].copy_from_slice(mc);
        Self(enc)
    }
}

impl crate::Encode<u16> for Insn {
    fn encode(&self) -> std::vec::Vec<u16> {
        let mut v = self.0.to_vec();
        v.truncate(<Insn as crate::Encode<u16>>::len(self));
        v
    }

    fn len(&self) -> usize {
        let sz_bytes: usize = (self.decode().1 + 1).try_into().unwrap();
        sz_bytes / 2
    }
}

impl crate::Branch for Insn {}

impl crate::subroutine::ShouldReturn for Insn {
    fn should_return(&self, offset: usize) -> Result<(), crate::StaticAnalysis<Self>> {
        if self.0[0] == 0x4e75 {
            StaticAnalysis::ok()
        } else {
            StaticAnalysis::err("ShouldReturn", Self::next_opcode, offset)
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> std::vec::Vec<u8> {
        let mut v: Vec<u8> = self.0.iter().flat_map(|&num| num.to_be_bytes()).collect();
        v.truncate(<Insn as crate::Encode<u8>>::len(self));
        v
    }

    fn len(&self) -> usize {
        (self.decode().1 + 1).try_into().unwrap()
    }
}

impl crate::Step for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn next(&mut self) -> crate::IterationResult {
        if self.0[0] >= data::LAST_LEGAL_OPCODE {
            Err(crate::StepError::End)
        } else {
            self.incr_at_offset(self.len_w() - 1);
            self.fixup()
        }
    }
}

impl Insn {
    /// Decodes the instruction and returns useful information representing its function, and its
    /// length
    pub fn decode(&self) -> (m68000::instruction::Instruction, u32) {
        let mut memory = self.0.as_slice();
        let mut memory = m68000::memory_access::MemoryIter {
            memory: &mut memory,
            next_addr: 0,
        };
        match m68000::instruction::Instruction::from_memory(&mut memory) {
            Ok(ins) => (ins, memory.next_addr),
            Err(e) => panic!("{e:?}"),
        }
    }

    fn len_b(&self) -> usize {
        (self.decode().1 + 1).try_into().unwrap()
    }

    fn len_w(&self) -> usize {
        self.len_b() / 2
    }

    fn fixup(&mut self) -> IterationResult {
        for i in data::RANGES_OF_ILLEGAL_OPCODES {
            if i.contains(&self.0[0]) {
                self.0[0] = i.end;
            }
        }
        Ok(())
    }

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

    #[test]
    fn skip_bad_instructions() {
        for rng in super::data::RANGES_OF_ILLEGAL_OPCODES {
            for opcode in rng {
                let i = Insn::new(&[opcode]);

                // Check that the `.fixup()` method actually changed the instruction
                let mut j = i.clone();
                j.fixup().unwrap();
                assert_ne!(i, j, "{i:?} should've fixed up");

                // check that calling `.fixup()` again does *not* change the instruction
                let mut k = j.clone();
                k.fixup().unwrap();
                assert_eq!(j, k, "{j:?} shouldn't've fixed up");
            }
        }
    }

    fn known_by_m68000_crate(insn: &Insn) -> bool {
        if format!("{insn}").starts_with("Unknown instruction") {
            return false;
        }
        if format!("{insn}").starts_with("ILLEGAL") {
            return false;
        }
        true
    }

    #[test]
    fn can_iterate_over_all_instructions() {
        use crate::Step;

        let mut i = Insn::first();

        while i.next().is_ok() {
            println!("{i:?}");
            let mut j = i;
            while j.decode().0.opcode == i.decode().0.opcode {
                j.next();
            }
            i.next_opcode();
            assert_eq!(j.decode().0.opcode, i.decode().0.opcode);
        }
    }

    #[test]
    fn no_unknown_instructions() {
        // if the instruction is unknown to the m68000 crate, then the disassembler prints
        // `Unknown instruction ...`. So this test makes sure that no such opcodes are found.
        use crate::Step;

        let mut i = Insn::first();

        while i.next_opcode().is_ok() {
            assert!(known_by_m68000_crate(&i), "{i:?}");
        }
    }

    #[ignore]
    #[test]
    fn no_duplicates() {
        // if one instruction has two possible encodings, then that's going to double the time that
        // the bruteforce search takes. So this test fails if it finds this kind of duplicate.
        use crate::Step;

        let mut i = Insn::first();

        while i.next_opcode().is_ok() {
            assert!(known_by_m68000_crate(&i), "{i:?}");
            println!("Checking for duplicates of {i:?}");
            let mut j = i.clone();
            while j.next_opcode().is_ok() {
                if format!("{i}") == format!("{j}") {
                    panic!("These two instructions both encode {i}: {i:?} and {j:?}");
                }
            }
        }
    }
}
