//! Module implementing a brute-force search

use crate::Candidate;
use crate::InstructionSet;
use crate::SearchFeedback;

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug, Default)]
pub struct BruteForceSearch<I: InstructionSet> {
    instruction_set: I,
    curr: Vec<I::Instruction>,
    maximum_length: usize,
}

impl<I: InstructionSet> SearchFeedback for BruteForceSearch<I> {
    fn score(&mut self, _: f32) {}
}

impl<I: InstructionSet> BruteForceSearch<I> {
    /// Creates a new `BruteForceSearch` from an `InstructionSet`. (That is, a bruteforce search
    /// over the search space defined by the instruction set).
    pub fn new(instruction_set: I, maximum_length: usize) -> BruteForceSearch<I> {
        BruteForceSearch {
            instruction_set,
            curr: vec![],
            maximum_length,
        }
    }
}

impl<I: InstructionSet> BruteForceSearch<I> {
    fn iterate(&mut self, offset: usize) {
        if offset >= self.curr.len() {
            // We've run off the current length of the vector, so append a new instruction iterator
            // and its first instruction.
            self.curr.push(self.instruction_set.first());
            return;
        }

        while self.instruction_set.next(&mut self.curr[offset]).is_some() {
            // If the altered program passes static analysis, then return
            if self.instruction_set.filter(&self.candidate()) {
                return;
            }
        }

        // We've exhausted all possibilities for this offset; try the next offset
        self.curr[offset] = self.instruction_set.first();
        self.iterate(offset + 1);
    }

    fn candidate(&self) -> Candidate<I::Instruction> {
        Candidate::new(self.curr.clone())
    }
}

impl<I> Iterator for BruteForceSearch<I>
where
    I: InstructionSet,
{
    type Item = Candidate<<I as InstructionSet>::Instruction>;

    fn next(&mut self) -> Option<Candidate<<I as InstructionSet>::Instruction>> {
        self.iterate(0);
        if self.curr.len() <= self.maximum_length {
            Some(self.candidate())
        } else {
            None
        }
    }
}
