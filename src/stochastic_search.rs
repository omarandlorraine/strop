//! Module for searching for solutions stochastically.

use crate::SearchFeedback;
use crate::{Candidate, Instruction, InstructionSet, Lcg};

/// A candidate program
#[derive(Clone, Debug)]
pub struct StochasticSearch<I: InstructionSet> {
    instruction_set: I,
    /// The program under current consideration.
    pub instructions: Candidate<I::Instruction>,
    fitness: Option<f32>,
    prng: Lcg,
}

impl<I: InstructionSet> SearchFeedback for StochasticSearch<I> {
    fn score(&mut self, score: f32) {
        self.fitness = Some(score);
    }
}

impl<I: InstructionSet> StochasticSearch<I> {
    /// returns a new `Candidate`
    pub fn new(instruction_set: I) -> Self {
        // Empty list of instructions
        let instructions = Candidate::<I::Instruction>::empty();

        // we don't have a valid fitness score yet
        let fitness = None;

        let prng = Lcg::new(rand::random());

        Self {
            instruction_set,
            instructions,
            fitness,
            prng,
        }
    }

    fn random_offset(&mut self) -> usize {
        let offset: usize = self.prng.next().unwrap().into();
        offset % self.instructions.instructions.len()
    }

    fn delete(&mut self) {
        // If the list of instructions contains at least one instruction, then delete one at
        // random.
        if !self.instructions.instructions.is_empty() {
            let offset = self.random_offset();
            self.instructions.instructions.remove(offset);
        }
    }

    fn insert(&mut self) {
        // Insert a randomly generated instruction at a random location in the program.
        let offset = if self.instructions.instructions.is_empty() {
            0
        } else {
            self.random_offset()
        };
        self.instructions.instructions.insert(offset, I::random());
    }

    fn swap(&mut self) {
        // If the program contains at least two instructions, then pick two at random and swap them
        // over.
        if self.instructions.instructions.len() > 2 {
            let offset_a = self.random_offset();
            let offset_b = self.random_offset();
            let instruction_a = self.instructions.instructions[offset_a];
            let instruction_b = self.instructions.instructions[offset_b];
            self.instructions.instructions[offset_a] = instruction_b;
            self.instructions.instructions[offset_b] = instruction_a;
        }
    }

    fn replace(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and swap it for something totally different.
        if !self.instructions.instructions.is_empty() {
            let offset = self.random_offset();
            self.instructions.instructions[offset] = I::random();
        }
    }

    fn mutate(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !self.instructions.instructions.is_empty() {
            let offset = self.random_offset();
            self.instructions.instructions[offset].mutate();
        }
    }

    /// Randomly mutates the `Candidate`
    pub fn random_mutation(&mut self) {
        let choice = self.prng.next().unwrap() % 5;

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
        None
    }
}
