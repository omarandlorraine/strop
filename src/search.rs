//! Module containing definitions of miscellaneous search strategies.

use crate::SearchFeedback;
use crate::{Candidate, InstructionSet, Instruction};

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

/// A very fast dead-code eliminator. Just tries removing each instruction in turn, and testing if
/// the resulting program is still correct.
pub fn quick_dead_code_eliminate<T: Instruction>(unoptimized: &Candidate<T>, score: fn(&Candidate<T>) -> bool) -> Candidate<T> {
    let mut optimized = unoptimized.clone();
    let mut getting_there = unoptimized.clone();

    let mut current_offset = 0usize;

    loop {
        if current_offset > getting_there.length() {
            return optimized;
        }
        getting_there.instructions.remove(current_offset);

        if (score)(&getting_there) {
            optimized = getting_there.clone();
        } else {
            getting_there = optimized.clone();
        }
        current_offset += 1;
    }
}
