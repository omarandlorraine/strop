use crate::Fixup;
use crate::StaticAnalysis;

/// A sequence of instructions
#[derive(Clone)]
pub struct Sequence<Instruction: crate::Instruction>(Vec<Instruction>);

impl<Instruction: crate::Instruction> std::fmt::Display for Sequence<Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in &self.0 {
            writeln!(f, "\t{i}")?;
        }
        Ok(())
    }
}

impl<Instruction: crate::Instruction> std::fmt::Debug for Sequence<Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for i in &self.0 {
            writeln!(f, "\t{i:?}")?;
        }
        Ok(())
    }
}

impl<Instruction: crate::Instruction> From<Vec<Instruction>> for Sequence<Instruction> {
    fn from(v: Vec<Instruction>) -> Self {
        Self(v)
    }
}

impl<Instruction: crate::Instruction> Default for Sequence<Instruction> {
    fn default() -> Self {
        Self::first()
    }
}

impl<Instruction: crate::Instruction> Sequence<Instruction> {
    /// Returns the first non-empty instruction sequence
    pub fn first() -> Self {
        Self(vec![Instruction::first()])
    }
    fn step_at(&mut self, offs: usize) {
        let mut offset = offs;
        loop {
            if offset == self.0.len() {
                self.0.push(Instruction::first());
                break;
            } else if self.0[offset].increment().is_err() {
                self.0[offset] = Instruction::first();
                offset += 1;
            } else {
                break;
            }
        }
    }

    /// Applies the fixup to the code sequence
    pub fn apply(&mut self, fixup: &crate::Fixup<Instruction>) {
        if (fixup.advance)(&mut self.0[fixup.offset]).is_err() {
            self.0[fixup.offset] = Instruction::first();
            self.step_at(fixup.offset + 1);
        }
    }

    /// Checks the last instruction in the sequence for static analysis
    pub fn check_last(
        &self,
        sa: fn(&Instruction) -> StaticAnalysis<Instruction>,
    ) -> StaticAnalysis<Instruction> {
        let offset = self.0.len() - 1;
        if let Err(Fixup {
            advance,
            offset: _,
            reason,
        }) = sa(&self.0[offset])
        {
            return Err(Fixup {
                advance,
                offset,
                reason,
            });
        }
        Ok(())
    }

    /// Checks all instructions for static analysis
    pub fn check_all(
        &self,
        sa: fn(&Instruction) -> StaticAnalysis<Instruction>,
    ) -> StaticAnalysis<Instruction> {
        crate::cull!(self.0, sa);
        for (offset, insn) in self.0.iter().enumerate() {
            if let Err(Fixup {
                advance,
                offset: _,
                reason,
            }) = sa(insn)
            {
                return Err(Fixup {
                    advance,
                    offset,
                    reason,
                });
            }
        }
        Ok(())
    }

    /// Checks all instructions except the last one for static analysis
    pub fn check_all_but_last(
        &self,
        sa: fn(&Instruction) -> StaticAnalysis<Instruction>,
    ) -> StaticAnalysis<Instruction> {
        for (offset, insn) in self.0.iter().take(self.0.len() - 1).enumerate() {
            if let Err(Fixup {
                advance,
                offset: _,
                reason,
            }) = sa(insn)
            {
                return Err(Fixup {
                    advance,
                    offset,
                    reason,
                });
            }
        }
        Ok(())
    }

    /// Steps through the search space in a manner similar to what a brute force search would do.
    pub fn increment(&mut self) {
        self.step_at(0);
    }
    fn random_offset(&self) -> usize {
        use rand::RngExt;
        let mut rng = rand::rng();
        if self.0.is_empty() {
            0
        } else {
            rng.random_range(0..self.0.len())
        }
    }
    /// Steps through the search space in a manner similar to what a random walk would do
    pub fn mutate(&mut self) {
        let choice: usize = rand::random_range(0..6);
        match choice {
            0 => {
                // maybe add a random instruction someplace in the sequence
                self.0.insert(self.random_offset(), Instruction::random());
            }
            1 => {
                // maybe delete a random instruction from the sequence
                if self.0.len() > 1 {
                    let offset = self.random_offset();
                    self.0.remove(offset);
                }
            }
            2 => {
                // maybe pick two instructions from the sequence at random and swap them over
                if self.len() > 2 {
                    let offset_a = self.random_offset();
                    let offset_b = self.random_offset();
                    self.0.swap(offset_a, offset_b);
                }
            }
            3 => {
                // maybe replace an instruction with something completely different
                if self.0.len() > 1 {
                    let offs = self.random_offset();
                    self.0[offs] = Instruction::random();
                }
            }
            _ => {
                // maybe mutate an existing instruction in place
                if self.0.len() > 1 {
                    let offs = self.random_offset();
                    self.0[offs].mutate();
                }
            }
        }
    }

    /// Disassembles the code, returning an instruction sequence.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut insns: Vec<Instruction> = vec![];

        let mut bytes: Vec<u8> = bytes.to_vec();

        while !bytes.is_empty() {
            let i = Instruction::from_bytes(&bytes)?;
            for _ in i.to_bytes() {
                bytes.remove(0);
            }
            insns.push(i);
        }
        Some(Self(insns))
    }

    /// Assembles the code, returning a raw byte sequence.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|i| i.to_bytes()).collect()
    }

    /// Returns the offset to the last instruction
    pub fn offset_to_last(&self) -> usize {
        self.0.len() - 1
    }
}

impl<Instruction: crate::Instruction> IntoIterator for Sequence<Instruction> {
    type Item = Instruction;
    type IntoIter = <Vec<Instruction> as IntoIterator>::IntoIter;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

impl<Instruction: crate::Instruction> std::ops::Deref for Sequence<Instruction> {
    type Target = [Instruction];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
