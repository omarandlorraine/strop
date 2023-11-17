//! Module for searching for solutions stochastically.

use crate::SearchFeedback;
use crate::{Candidate, Instruction, InstructionSet};

/// A candidate program
#[derive(Clone, Debug)]
pub struct StochasticSearch<I: InstructionSet> {
    parent: Candidate<I::Instruction>,
    child: Candidate<I::Instruction>,
    parent_score: f32,
    child_score: f32,
    instruction_set: I,
}

impl<I: InstructionSet> SearchFeedback for StochasticSearch<I> {
    fn score(&mut self, score: f32) {
        self.child_score = score.abs();
    }
}

impl<I: InstructionSet> StochasticSearch<I> {
    /// returns a new `Candidate`
    pub fn new(instruction_set: I) -> Self {
        // Empty list of instructions
        let parent = Candidate::<I::Instruction>::empty();
        let child = Candidate::<I::Instruction>::empty();
        let parent_score = f32::MAX;
        let child_score = f32::MAX;

        Self {
            parent,
            parent_score,
            child,
            child_score,
            instruction_set,
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
        self.child
            .instructions
            .insert(offset, self.instruction_set.random());
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
            self.child.instructions[offset] = self.instruction_set.random();
        }
    }

    fn mutate(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !self.child.instructions.is_empty() {
            let offset = self.random_offset();
            self.instruction_set
                .mutate(&mut self.child.instructions[offset]);
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
    }
}

impl<I: InstructionSet> Iterator for StochasticSearch<I> {
    type Item = Candidate<<I as InstructionSet>::Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
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
