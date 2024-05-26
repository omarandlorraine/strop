//! Module defining the stochastic search algorithm

use crate::Candidate;
use crate::Fixup;
use crate::Instruction;
use crate::SearchAlgorithm;

fn random_mutation<I: Instruction>(candidate: &mut Candidate<I>) {
    fn random_offset<I: Instruction>(candidate: &Candidate<I>) -> usize {
        use rand::Rng;
        rand::thread_rng().gen_range(0..candidate.instructions.len())
    }

    fn delete<I: Instruction>(candidate: &mut Candidate<I>) {
        // If the list of instructions contains at least one instruction, then delete one at
        // random.
        if !candidate.instructions.is_empty() {
            let offset = random_offset(candidate);
            candidate.instructions.remove(offset);
        }
    }

    fn insert<I: Instruction>(candidate: &mut Candidate<I>) {
        // Insert a randomly generated instruction at a random location in the program.
        let offset = if candidate.instructions.is_empty() {
            0
        } else {
            random_offset(candidate)
        };
        candidate.instructions.insert(offset, I::random());
    }

    fn swap<I: Instruction>(candidate: &mut Candidate<I>) {
        // If the program contains at least two instructions, then pick two at random and swap them
        // over.
        if candidate.instructions.len() > 2 {
            let offset_a = random_offset(candidate);
            let offset_b = random_offset(candidate);
            let instruction_a = candidate.instructions[offset_a];
            let instruction_b = candidate.instructions[offset_b];
            candidate.instructions[offset_a] = instruction_b;
            candidate.instructions[offset_b] = instruction_a;
        }
    }

    fn replace<I: Instruction>(candidate: &mut Candidate<I>) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and swap it for something totally different.
        if !candidate.instructions.is_empty() {
            let offset = random_offset(candidate);
            candidate.instructions[offset] = I::random();
        }
    }

    fn mutate<I: Instruction>(candidate: &mut Candidate<I>) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !candidate.instructions.is_empty() {
            let offset = random_offset(candidate);
            candidate.instructions[offset].mutate();
        }
    }

    use rand::Rng;
    let choice = rand::thread_rng().gen_range(0..5);

    match choice {
        0 => {
            delete(candidate);
        }
        1 => {
            insert(candidate);
        }
        2 => {
            swap(candidate);
        }
        3 => {
            replace(candidate);
        }
        4 => {
            mutate(candidate);
        }
        _ => panic!(),
    }
    if candidate.length() == 0 {
        random_mutation(candidate);
    }
}

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

    fn start_from(&mut self, point: &Candidate<Self::Item>) {
        self.parent = point.clone();
        self.child = point.clone();
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

        let r = self.child.clone();

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
        random_mutation(&mut self.child);
        Some(r)
    }

    fn peek(&self) -> &Candidate<I> {
        &self.child
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
}

impl<I: Instruction> Default for StochasticSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a program by stochastic approximation to a correctness function
#[derive(Clone, Default, Debug)]
pub struct StochasticDeadCodeEliminator<I: Instruction> {
    parent: Candidate<I>,
    child: Candidate<I>,
}

impl<I: Instruction> StochasticDeadCodeEliminator<I> {
    /// Creates a new `StochasticDeadCodeEliminator`
    pub fn new() -> Self {
        Self {
            parent: Candidate::<I>::default(),
            child: Candidate::<I>::default(),
        }
    }

    fn reset(&mut self) {
        // call this when the search has probably wondered off into the weeds and needs to restart
        self.child = self.parent.clone();
    }

    fn forward(&mut self) {
        // call this when the search has provably made some progress
        self.parent = self.child.clone();
        self.parent.disassemble();
    }
}

impl<I: Instruction> SearchAlgorithm for StochasticDeadCodeEliminator<I> {
    type Item = I;

    fn start_from(&mut self, original: &Candidate<I>) {
        self.parent = original.clone();
        self.child = original.clone();
    }

    fn score(&mut self, score: f32) {
        if score != 0.0 {
            self.reset();
        } else {
            self.forward();
        }
    }

    fn peek(&self) -> &Candidate<Self::Item> {
        &self.child
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

        for _ in 0..rand::thread_rng().gen_range(0..=5) {
            random_mutation(&mut self.child);
        }

        // if the child is now not shorter, then go back to the parent
        if self.child.encode().len() > self.parent.encode().len() {
            self.reset();
        }

        Some(self.child.clone())
    }
}
