//! Module containing definitions of miscellaneous search strategies.

use crate::Fixup;
use crate::SearchAlgorithm;
use crate::{Candidate, Instruction};

/// Iterates across the entire search space, shortest programs first.
#[derive(Clone, Debug)]
pub struct BruteForceSearch<I: Instruction + PartialOrd + PartialEq, F: Fixup<I>> {
    curr: Candidate<I>,
    max_length: usize,
    fixup: F,
}

impl<I: Instruction + std::cmp::PartialOrd, F: Fixup<I> + Clone> SearchAlgorithm for BruteForceSearch<I, F> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn replace<G: Fixup<I>>(&mut self, offset: usize, fixup: G) -> bool {
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

impl<I: Instruction + std::cmp::PartialOrd + std::cmp::PartialEq, F: Fixup<I> + Clone> BruteForceSearch<I, F> {
    /// Returns a new `BruteForceSearch<I>`
    pub fn new(fixup: F) -> Self {
        Self {
            curr: Candidate::empty(),
            max_length: usize::MAX,
            fixup,
        }
    }

    /// Returns a new `BruteForceSearch<I>`, which stops iterating when it hits a program of length
    /// `max_length`.
    pub fn new_with_length(fixup: F, max_length: usize) -> Self {
        Self {
            curr: Candidate::empty(),
            max_length,
            fixup,
        }
    }

    fn fcheck(&mut self, offset: usize) {
        let previous = self.curr.instructions[offset];
        if self.fixup.check(previous) {
            if let Some(next) = self.fixup.next(previous) {
                println!("swapping {} for {}", previous, next);
                self.curr.instructions[offset] = next;
            } else {
                self.curr.instructions[offset] = I::first();
                self.iterate(offset + 1);
            }
        }
    }

    fn iterate(&mut self, offset: usize) {
        if offset >= self.curr.instructions.len() {
            // We've run off the current length of the vector, so append another instruction
            self.curr.instructions.push(I::first());
            self.fcheck(offset);
            return;
        }

        if let Some(insn) = self.curr.instructions[offset].increment() {
            self.curr.instructions[offset] = insn;
            self.fcheck(offset);
            return;
        }

        // We've exhausted all possibilities for this offset; try the next offset
        self.curr.instructions[offset] = I::first();
        self.fcheck(offset);
        self.iterate(offset + 1);
    }
}
