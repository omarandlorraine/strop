//! The Z80 backend (can of course also be used to generate code for the Intel 8080 or the SM83).
#![allow(dead_code)] // TODO: enable this lint

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::Candidate;
pub use crate::SearchAlgorithm;
pub use crate::SingleInstruction;
pub use instruction_set::Z80Instruction;
pub use testers::Z88dkFastCall;

const RET: SingleInstruction<Z80Instruction> =
    SingleInstruction(Z80Instruction::new([0xc9, 0, 0, 0, 0]));
const RETI: SingleInstruction<Z80Instruction> =
    SingleInstruction(Z80Instruction::new([0xed, 0x4d, 0, 0, 0]));
const RETN: SingleInstruction<Z80Instruction> =
    SingleInstruction(Z80Instruction::new([0xed, 0x45, 0, 0, 0]));

#[derive(Debug)]
struct Subroutine<S: SearchAlgorithm<Item = Z80Instruction>>(S);

impl<S: SearchAlgorithm<Item = Z80Instruction>> SearchAlgorithm for Subroutine<S> {
    type Item = Z80Instruction;

    fn score(&mut self, score: f32) {
        self.0.score(score)
    }

    fn replace<F: crate::Fixup<Self::Item>>(&mut self, offset: usize, fixup: F) -> bool {
        self.0.replace(offset, fixup)
    }

    fn generate(&mut self) -> Option<Candidate<Z80Instruction>> {
        self.0.generate();
        self.sanity();
        Some(self.peek().clone())
    }

    fn peek(&self) -> &Candidate<Z80Instruction> {
        self.0.peek()
    }
}

impl<S: SearchAlgorithm<Item = Z80Instruction>> Subroutine<S> {
    pub fn new(inner: S) -> Self {
        let mut r = Self(inner);
        r.sanity();
        r
    }

    fn sanity(&mut self) {
        // ensures the next Candidate to be returned ends in a RET instruction
        loop {
            let r = self.0.peek();

            if r.instructions.is_empty() {
                // no instruction in the candidate; generate another one
                self.0.generate();
                continue;
            }

            if self.0.replace(r.last_offset(), RET) {
                // the candidate didn't end in a RET instruction, but a new candidate has been
                // generated, so go back and consider it
                continue;
            }

            break;
        }
    }
}
