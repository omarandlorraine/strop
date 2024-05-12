//! Module containing definitions of miscellaneous search strategies.

use crate::Fixup;
use crate::Fitness;
use crate::SearchAlgorithm;
use crate::{Candidate, Instruction};

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug)]
pub struct BruteForceSearch<I: Instruction + PartialOrd + PartialEq> {
    curr: Vec<I>,
    max_length: usize,
}

impl<I: Instruction + std::cmp::PartialOrd> SearchAlgorithm for BruteForceSearch<I> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn fitness(&mut self, _cand: &Candidate<Self::Item>) -> Fitness {
        Fitness::Passes(0.0)
    }

    fn replace<F: Fixup<I>>(&mut self, offset: usize, fixup: F) {
        let orig = self.curr[offset];
        if fixup.check(orig) {
            if let Some(next) = fixup.next(orig){
                assert!(orig < next);
                self.curr[offset] = next;
            } else {
                self.curr[offset] = I::first();
                self.curr.push(I::first());
            }
        }
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        self.iterate(0);
        if self.candidate().length() < self.max_length {
            Some(self.candidate())
        } else {
            None
        }
    }
}

impl<I: Instruction + std::cmp::PartialOrd + std::cmp::PartialEq> BruteForceSearch<I> {
    /// Returns a new `BruteForceSearch<I>`
    pub fn new() -> Self {
        Self { curr: vec![], max_length: usize::MAX }
    }

    /// Returns a new `BruteForceSearch<I>`, which stops iterating when it hits a program of length
    /// `max_length`.
    pub fn new_with_length(max_length: usize) -> Self {
        Self { curr: vec![], max_length }
    }

    fn iterate(&mut self, offset: usize) {
        if offset >= self.curr.len() {
            // We've run off the current length of the vector, so append another instruction
            self.curr.push(I::first());
            return;
        }

        if let Some(insn) = self.curr[offset].increment() {
            self.curr[offset] = insn;
            return;
        }

        // We've exhausted all possibilities for this offset; try the next offset
        self.curr[offset] = I::first();
        self.iterate(offset + 1);
    }

    fn candidate(&self) -> Candidate<I> {
        Candidate::new(self.curr.clone())
    }
}

impl<I: Instruction + PartialOrd + PartialEq> Default for BruteForceSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}
