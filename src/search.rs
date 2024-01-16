//! Module containing definitions of miscellaneous search strategies.

use crate::SearchAlgorithm;
use crate::{Candidate, Instruction};

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

    fn score(&mut self, score: f32) {
        self.child_score = score.abs();
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        use rand::random;
        self.child.instructions[offset] = if random() { instruction.unwrap_or_else(I::random) } else { I::random() }
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
    }
}

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug)]
pub struct BruteForceSearch<I: Instruction> {
    curr: Vec<I>,
}

impl<I: Instruction> SearchAlgorithm for BruteForceSearch<I> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        if let Some(instruction) = instruction {
            self.curr[offset] = instruction
        } else {
            self.iterate(offset);
        }
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        self.iterate(0);
        Some(self.candidate())
    }
}

impl<I: Instruction> BruteForceSearch<I> {
    /// Returns a new BruteForceSearch<I>
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

    /// Limits the length of the generated programs
    pub fn limit_length(self, length: usize) -> LengthLimitedSearch<Self, I> {
        LengthLimitedSearch {
            inner: self,
            length,
        }
    }
}

#[derive(Debug)]
/// A SearchAlgorithm that rejects any programs having a longer length than the one specified.
pub struct LengthLimitedSearch<S: SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
    length: usize,
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> SearchAlgorithm for LengthLimitedSearch<S, I> {
    type Item = I;

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>) {
        self.inner.replace(offset, instruction);
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        let cand = self.inner.generate().unwrap();
        if cand.instructions.len() <= self.length {
            Some(cand)
        } else {
            None
        }
    }
}

/// A static analysis pass that rejects any programs that are not basic blocks; (i.e., no
/// instruction in the program may influence the control flow, except the last instruction).
/// Influencing the control flow here means, branches, jumps, subroutines, returns, etc.
#[derive(Debug)]
pub struct BasicBlock<S: ?Sized + SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
}

impl<S, I> BasicBlock<S, I> 
where S: Sized + SearchAlgorithm<Item = I>, I: Instruction
{
    /// Creates a new BasicBlock from another search algorithm.
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> SearchAlgorithm for BasicBlock<S, I> {
    type Item = I;

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>) {
        self.inner.replace(offset, instruction);
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        use crate::SearchCull::SkipTo;

        'outer: while let Some(cand) = self.inner.generate() {
            for (offset, instruction) in cand.instructions.iter().take(cand.instructions.len() - 1).enumerate() {
                if let SkipTo(i) = instruction.cull_flow_control() {
                    self.inner.replace(offset, i);
                    continue 'outer;
                }
            }
            return Some(cand);
        }
        None
    }
}

/// A static analysis pass which rejects any flow control instructions
#[derive(Debug)]
pub struct NoFlowControl<S: ?Sized + SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
}

impl<S, I> NoFlowControl<S, I> 
where S: Sized + SearchAlgorithm<Item = I>, I: Instruction
{
    /// Creates a new NoFlowControl from another search algorithm.
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> SearchAlgorithm for NoFlowControl<S, I> {
    type Item = I;

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>) {
        self.inner.replace(offset, instruction);
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        use crate::SearchCull::SkipTo;

        'outer: while let Some(cand) = self.inner.generate() {
            for (offset, instruction) in cand.instructions.iter().enumerate() {
                if let SkipTo(i) = instruction.cull_flow_control() {
                    self.inner.replace(offset, i);
                    continue 'outer;
                }
            }
            return Some(cand);
        }
        None
    }
}

/// Random dead-code eliminator
#[derive(Clone, Debug)]
pub struct DeadCodeEliminator<I: Instruction> {
    parent: Candidate<I>,
    child: Candidate<I>,
}

impl<I: Instruction> SearchAlgorithm for DeadCodeEliminator<I> {
    type Item = I;
    fn score(&mut self, score: f32) {
        if score != 0.0 {
            self.child = self.parent.clone();
        } else {
            self.parent = self.child.clone();
        }
    }

    fn replace(&mut self, _offset: usize, _instruction: Option<I>) {
        self.child = self.parent.clone();
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        use rand::random;
        if random() {
            self.delete();
        }
        Some(self.child.clone())
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
