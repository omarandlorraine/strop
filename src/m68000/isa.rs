use crate::{IterationResult, StepError};

/// Represents one 68000 machine code instruction
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Insn(pub(crate) [u16; 5]);

impl Insn {
    /// Increments the first word of the instruction, and resets all the rest to zero
    pub fn next_opcode(&mut self) -> IterationResult {
        if self.0[0] > 0xe000 {
            return Err(StepError::End);
        }
        self.0[0] += 1;
        self.0[1] = 0;
        self.0[2] = 0;
        self.0[3] = 0;
        self.0[4] = 0;
        Ok(())
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

    fn next(&mut self) -> IterationResult {
        let mut offs = self.len_w() - 1;
        if self.0[0] > 0xe000 {
            return Err(StepError::End);
        }
        loop {
            if let Some(new_val) = self.0[offs].checked_add(1) {
                self.0[offs] = new_val;
                self.fixup();
                return Ok(());
            } else {
                if offs == 0 {
                    return Err(StepError::End);
                }
                self.0[offs] = 0;
                offs -= 1;
            }
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

    fn valid_encoding(&self) -> bool {
        let diss = format!("{self}");
        !(diss.contains("Unknown") || diss.contains("ILLEGAL"))
    }

    fn fixup(&mut self) {
        while !self.valid_encoding() {
            self.0[0] += 1;
        }
    }
}
