//! Module containing definitions of miscellaneous search strategies.

use crate::SearchFeedback;
use crate::{Candidate, Instruction};

/// Generates a program by stochastic approximation to a correctness function
#[derive(Clone, Debug)]
pub struct StochasticSearch<I: Instruction> {
    parent: Candidate<I>,
    child: Candidate<I>,
    parent_score: f32,
    child_score: f32,
}

impl<I: Instruction> SearchFeedback<I> for StochasticSearch<I> {
    fn score(&mut self, score: f32) {
        self.child_score = score.abs();
    }

    fn replace(&mut self, offset: usize, instruction: I) {
        use rand::random;
        self.child.instructions[offset] = if random() {
            instruction
        } else {
            I::random()
        }
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
    }
}

impl<I: Instruction> Iterator for StochasticSearch<I> {
    type Item = Candidate<I>;

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

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug)]
pub struct BruteForceSearch<I: Instruction> {
    curr: Vec<I>,
}

impl<I: Instruction> SearchFeedback<I> for BruteForceSearch<I> {
    fn score(&mut self, _: f32) {}

    fn replace(&mut self, offset: usize, instruction: I) {
        self.curr[offset] = instruction
    }
}

impl<I: Instruction> BruteForceSearch<I> {
    pub fn new() -> Self {
        Self { curr: vec![] }
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

impl<I> Iterator for BruteForceSearch<I>
where
    I: Instruction,
{
    type Item = Candidate<I>;

    fn next(&mut self) -> Option<Candidate<I>> {
        self.iterate(0);
        Some(self.candidate())
    }
}

/// Random dead-code eliminator
#[derive(Clone, Debug)]
pub struct DeadCodeEliminator<I: Instruction> {
    parent: Candidate<I>,
    child: Candidate<I>,
}

impl<I: Instruction> SearchFeedback<I> for DeadCodeEliminator<I> {
    fn score(&mut self, score: f32) {
        if score != 0.0 {
            self.child = self.parent.clone();
        } else {
            self.parent = self.child.clone();
        }
    }

    fn replace(&mut self, _offset: usize, _instruction: I) {
        self.child = self.parent.clone();
    }
}

impl<I: Instruction> DeadCodeEliminator<I> {
    /// returns a new `Candidate`
    pub fn new(unoptimized: &Candidate<I>) -> Self {
        Self {
            parent: unoptimized.clone(),
            child: unoptimized.clone(),
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
}

impl<I: Instruction> Iterator for DeadCodeEliminator<I> {
    type Item = Candidate<I>;

    fn next(&mut self) -> Option<Self::Item> {
        use rand::random;
        if random() {
            self.delete();
        }
        Some(self.child.clone())
    }
}

impl<I: Instruction> Default for StochasticSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Instruction> Default for BruteForceSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}
