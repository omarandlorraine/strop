//! Module defining the stochastic search algorithm

use crate::Candidate;
use crate::Fitness;
use crate::Instruction;
use crate::SearchAlgorithm;
use crate::Fixup;

/// Generates a program by stochastic approximation to a correctness function
#[derive(Clone, Debug)]
pub struct StochasticSearch<I: Instruction> {
    parent: Candidate<I>,
    child: Candidate<I>,
    parent_score: f32,
    child_score: f32,
}

impl<I: Instruction> SearchAlgorithm for StochasticSearch<I> {
    type Item = I;

    fn fitness(&mut self, _cand: &Candidate<Self::Item>) -> Fitness {
        Fitness::Passes(0.0)
    }

    fn score(&mut self, score: f32) {
        self.child_score = score.abs();
    }

    fn replace<F: Fixup<I>>(&mut self, offset: usize, fixup: F) -> bool {
        let orig = self.child.instructions[offset];
        if fixup.check(orig) {
            self.child.instructions[offset] = fixup.random(orig);
            true
        } else {
            false
        }
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        use rand::Rng;

        if self.child_score < self.parent_score {
            // The child is better than the parent, so definitely remember that
            self.parent_score = self.child_score;
            self.parent = self.child.clone();
        } else if self.child_score > self.parent_score {
            // The child is worse that the parent.
            let num = rand::thread_rng().gen_range(0.0..self.child_score);

            if num > self.parent_score {
                // Maybe the search has wandered off into the weeds. Try going back to the parent
                self.child = self.parent.clone();
                self.child_score = self.parent_score;
            }
        }
        self.random_mutation();
        Some(self.child.clone())
    }
}

impl<I: Instruction> StochasticSearch<I> {
    /// returns a new `Candidate`
    pub fn new() -> Self {
        // Empty list of instructions
        let parent = Candidate::<I>::empty();
        let child = Candidate::<I>::empty();
        let parent_score = f32::MAX;
        let child_score = f32::MAX;

        Self {
            parent,
            parent_score,
            child,
            child_score,
        }
    }

    fn random_offset(&mut self) -> usize {
        use rand::Rng;
        rand::thread_rng().gen_range(0..self.child.instructions.len())
    }

    fn delete(&mut self) {
        // If the list of instructions contains at least one instruction, then delete one at
        // random.
        if !self.child.instructions.is_empty() {
            let offset = self.random_offset();
            self.child.instructions.remove(offset);
        }
    }

    fn insert(&mut self) {
        // Insert a randomly generated instruction at a random location in the program.
        let offset = if self.child.instructions.is_empty() {
            0
        } else {
            self.random_offset()
        };
        self.child.instructions.insert(offset, I::random());
    }

    fn swap(&mut self) {
        // If the program contains at least two instructions, then pick two at random and swap them
        // over.
        if self.child.instructions.len() > 2 {
            let offset_a = self.random_offset();
            let offset_b = self.random_offset();
            let instruction_a = self.child.instructions[offset_a];
            let instruction_b = self.child.instructions[offset_b];
            self.child.instructions[offset_a] = instruction_b;
            self.child.instructions[offset_b] = instruction_a;
        }
    }

    fn replace(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and swap it for something totally different.
        if !self.child.instructions.is_empty() {
            let offset = self.random_offset();
            self.child.instructions[offset] = I::random();
        }
    }

    fn mutate(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !self.child.instructions.is_empty() {
            let offset = self.random_offset();
            self.child.instructions[offset].mutate();
        }
    }

    /// Randomly mutates the `Candidate`
    pub fn random_mutation(&mut self) {
        use rand::Rng;
        let choice = rand::thread_rng().gen_range(0..5);

        match choice {
            0 => self.delete(),
            1 => self.insert(),
            2 => self.swap(),
            3 => self.replace(),
            4 => self.mutate(),
            _ => panic!(),
        }
        if self.child.length() == 0 {
            self.random_mutation();
        }
    }
}

impl<I: Instruction> Default for StochasticSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}
