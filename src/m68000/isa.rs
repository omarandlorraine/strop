/// Represents one 68000 machine code instruction
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Insn(pub(crate) [u16; 5]);

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

impl crate::Iterable for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn step(&mut self) -> bool {
        let mut offs = self.len_w() - 1;
        if self.0[0] > 0xe000 {
            return false;
        }
        loop {
            if let Some(new_val) = self.0[offs].checked_add(1) {
                self.0[offs] = new_val;
                self.fixup();
                return true;
            } else {
                if offs == 0 {
                    return false;
                }
                self.0[offs] = 0;
                offs -= 1;
            }
        }
    }

    fn stride(&mut self) -> bool {
        if self.0[0] > 0xe000 {
            false
        } else {
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
            true
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
            Err(e) => panic!("{:?}", e),
        }
    }

    fn len_b(&self) -> usize {
        (self.decode().1 + 1).try_into().unwrap()
    }

    fn len_w(&self) -> usize {
        self.len_b() / 2
    }

    fn valid_encoding(&self) -> bool {
        let diss = format!("{}", self);
        !(diss.contains("Unknown") || diss.contains("ILLEGAL"))
    }

    fn fixup(&mut self) {
        while !self.valid_encoding() {
            self.0[0] += 1;
        }
    }

    /// Bumps the opcode, and sets all subsequent words back to zero (handy for if static analysis
    /// determines an opcode to be unsuitable)
    pub fn bump_opcode(&self) -> crate::ConstraintViolation<Self> {
        if self.0[0] > 0xefbe {
            // Apparently there are no opcodes higher than 0xEFBE.
            crate::ConstraintViolation::Violation
        } else {
            let mut i = Insn([self.0[0] + 1, 0, 0, 0, 0]);
            i.fixup();
            crate::ConstraintViolation::ReplaceWith(i)
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_opcodes() {
        use crate::ConstraintViolation;
        use crate::Iterable;
        let mut insn = super::Insn::first();
        loop {
            assert!(
                insn.len_b() > 1,
                "{:?}, length in bytes: {}",
                insn,
                insn.len_b()
            );
            let diss = format!("{}", insn);
            assert!(!insn.decode().1 > 0);
            assert!(!diss.contains("Unknown"), "{:?}", insn);
            assert!(!diss.contains("ILLEGAL"), "{:?}", insn);

            match insn.bump_opcode() {
                ConstraintViolation::Ok => {}
                ConstraintViolation::ReplaceWith(me) => insn = me,
                ConstraintViolation::Violation => break,
            }
        }
    }
}
