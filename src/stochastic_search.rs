//! Module for searching for solutions stochastically.

use crate::{Candidate, Instruction, Lcg, Test};

/// A candidate program
#[derive(Clone, Debug)]
pub struct StochasticSearch<I: Instruction, T: Test<I>> {
    /// The program under current consideration.
    pub instructions: Candidate<I>,
    tester: T,
    fitness: Option<f64>,
    prng: Lcg,
}

impl<I: Instruction, T: Test<I>> StochasticSearch<I, T> {
    /// returns a new `Candidate`
    pub fn new(tester: T) -> Self {
        // Empty list of instructions
        let instructions = Candidate::<I>::empty();

        // we don't have a valid fitness score yet
        let fitness = None;

        let prng = Lcg::new(rand::random());

        Self {
            instructions,
            tester,
            fitness,
            prng,
        }
    }

    /// Recomputes the `Candidate`'s fitness
    pub fn score(&mut self) -> f64 {
        // if the fitness score has been invalidated, then recompute it
        if self.fitness.is_none() {
            self.fitness = Some(self.tester.run(&self.instructions));
        }
        self.fitness.unwrap()
    }

    fn invalidate_fitness_score(&mut self) {
        self.fitness = None;
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
            self.invalidate_fitness_score();
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
        self.invalidate_fitness_score();
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
            self.invalidate_fitness_score();
        }
    }

    fn replace(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and swap it for something totally different.
        if !self.instructions.instructions.is_empty() {
            let offset = self.random_offset();
            self.instructions.instructions[offset] = I::random();
            self.invalidate_fitness_score();
        }
    }

    fn mutate(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !self.instructions.instructions.is_empty() {
            let offset = self.random_offset();
            self.instructions.instructions[offset].mutate();
            self.invalidate_fitness_score();
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
