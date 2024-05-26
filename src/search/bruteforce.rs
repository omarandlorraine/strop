//! Module containing definitions of miscellaneous search strategies.

use crate::Fixup;
use crate::SearchAlgorithm;
use crate::{Candidate, Instruction};

/// Iterates across the entire search space, shortest programs first.
#[derive(Clone, Debug)]
pub struct BruteForceSearch<I: Instruction + PartialOrd + PartialEq> {
    curr: Candidate<I>,
    max_length: usize,
}

impl<I: Instruction + std::cmp::PartialOrd> SearchAlgorithm for BruteForceSearch<I> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn replace<F: Fixup<I>>(&mut self, offset: usize, fixup: F) -> bool {
        let orig = self.curr.instructions[offset];
        if fixup.check(orig) {
            if let Some(next) = fixup.next(orig) {
                assert!(orig < next);
                self.curr.instructions[offset] = next;
            } else {
                self.curr.instructions[offset] = I::first();
                self.curr.instructions.push(I::first());
            }
            true
        } else {
            false
        }
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        self.iterate(0);
        if self.curr.length() < self.max_length {
            Some(self.curr.clone())
        } else {
            None
        }
    }

    fn peek(&self) -> &Candidate<I> {
        &self.curr
    }

    fn start_from(&mut self, point: Candidate<I>) {
        self.curr = point;
    }
}

impl<I: Instruction + std::cmp::PartialOrd + std::cmp::PartialEq> BruteForceSearch<I> {
    /// Returns a new `BruteForceSearch<I>`
    pub fn new() -> Self {
        Self {
            curr: Candidate::empty(),
            max_length: usize::MAX,
        }
    }

    /// Returns a new `BruteForceSearch<I>`, which stops iterating when it hits a program of length
    /// `max_length`.
    pub fn new_with_length(max_length: usize) -> Self {
        Self {
            curr: Candidate::empty(),
            max_length,
        }
    }

    fn iterate(&mut self, offset: usize) {
        if offset >= self.curr.instructions.len() {
            // We've run off the current length of the vector, so append another instruction
            self.curr.instructions.push(I::first());
            return;
        }

        if let Some(insn) = self.curr.instructions[offset].increment() {
            self.curr.instructions[offset] = insn;
            return;
        }

        // We've exhausted all possibilities for this offset; try the next offset
        self.curr.instructions[offset] = I::first();
        self.iterate(offset + 1);
    }
}

impl<I: Instruction + PartialOrd + PartialEq> Default for BruteForceSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}
