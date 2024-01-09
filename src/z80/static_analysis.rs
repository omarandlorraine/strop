//! Module containing miscellaneous Z80 specific static analysis passes.

use crate::z80::Z80Instruction;
use crate::Candidate;
use crate::SearchAlgorithm;

/// Static analysis pass ensuring compatibility with the Intel 8080, by filtering away any use of
/// Z80-specific opcodes and prefixes
#[derive(Debug)]
pub struct I8080Compatible<S: SearchAlgorithm<Item = Z80Instruction>> {
    inner: S,
}

impl<S> I8080Compatible<S>
where
    S: SearchAlgorithm<Item = Z80Instruction>,
{
    /// Creates an I8080Compatible from a SearchAlgorithm
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> SearchAlgorithm for I8080Compatible<S>
where
    S: SearchAlgorithm<Item = Z80Instruction>,
{
    type Item = Z80Instruction;

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Self::Item) {
        self.inner.replace(offset, instruction);
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        use crate::Instruction;

        'outer: while let Some(c) = self.inner.generate() {
            for (offset, insn) in c.instructions.iter().enumerate() {
                let opcode = insn.encode()[0];
                if matches!(
                    opcode,
                    0x08 | 0x10
                        | 0x18
                        | 0x20
                        | 0x28
                        | 0x30
                        | 0x38
                        | 0xd9
                        | 0xcb
                        | 0xed
                        | 0xdd
                        | 0xfd
                ) {
                    // So one of the opcodes has an opcode which is not valid on the Intel 8080.
                    // Replace it, and try getting another candidate.
                    self.inner
                        .replace(offset, Z80Instruction::new([opcode + 1, 0, 0, 0, 0]));
                    continue 'outer;
                }
            }

            // We didn't find a Z80-only opcode, so return this candidate
            return Some(c);
        }
        None
    }
}
