//! Module containing definitions of miscellaneous search strategies.

use crate::Compatibility;
use crate::Linkage;
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
        self.child.instructions[offset] = if random() {
            instruction.unwrap_or_else(I::random)
        } else {
            I::random()
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
    }
}

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug)]
pub struct BruteForceSearch<I: Instruction + PartialOrd + PartialEq> {
    curr: Vec<I>,
}

impl<I: Instruction  + std::cmp::PartialOrd> SearchAlgorithm for BruteForceSearch<I> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        if let Some(instruction) = instruction {
            assert!(self.curr[offset] < instruction);
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

impl<I: Instruction + std::cmp::PartialOrd + std::cmp::PartialEq> BruteForceSearch<I> {
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

/// Stochastic dead-code eliminator
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

impl<I: Instruction + PartialOrd + PartialEq> Default for BruteForceSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// A static analysis pass which selects instructions for compatibility with particular CPU
/// variants.
#[derive(Debug)]
pub struct CompatibilitySearch<S: SearchAlgorithm<Item = I>, I: Instruction, C: Compatibility<I>> {
    inner: S,
    compatibility: C,
}

impl<S, I, C> CompatibilitySearch<S, I, C>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
    C: Compatibility<I>,
{
    /// Creates a new NoFlowControl from another search algorithm.
    pub fn new(inner: S, compatibility: C) -> Self {
        Self {
            inner,
            compatibility,
        }
    }
}

impl<S, I, C> SearchAlgorithm for CompatibilitySearch<S, I, C>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
    C: Compatibility<I>,
{
    type Item = I;
    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        use crate::SearchCull;

        'outer: loop {
            let candidate = self.inner.generate()?;
            for (offset, insn) in candidate.instructions.iter().enumerate() {
                match self.compatibility.check(insn) {
                    SearchCull::Okay => {}
                    SearchCull::SkipTo(i) => {
                        self.inner.replace(offset, i);
                        continue 'outer;
                    }
                }
            }
            return Some(candidate);
        }
    }
}

/// A static analysis pass for ensuring that subroutines have the correct prologues/epilogues
#[derive(Debug)]
pub struct LinkageSearch<S: SearchAlgorithm<Item = I>, I: Instruction, L: Linkage<S, I>> {
    inner: S,
    linkage: L,
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction, L: Linkage<S, I>> LinkageSearch<S, I, L> {
    /// Constructs a new LinkageSearch object, from an inner search algorithm, and some type
    /// implementing the `Linkage` trait, representing the prologue/epilogue details.
    pub fn new(inner: S, linkage: L) -> Self {
        Self { inner, linkage }
    }
}

impl<S, I, L> SearchAlgorithm for LinkageSearch<S, I, L>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
    L: Linkage<S, I>,
{
    type Item = I;
    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        loop {
            let candidate = self.inner.generate()?;
            if self.linkage.check(&mut self.inner, &candidate) {
                return Some(candidate);
            }
        }
    }
}

/// A static analysis pass for ensuring that programs do not access memory outside of the allow
/// ranges.
#[derive(Debug)]
pub struct MemoryAccessSearch<S: SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
    ranges: Vec<core::ops::Range<u16>>,
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> MemoryAccessSearch<S, I> {
    /// Constructs a new LinkageSearch object, from an inner search algorithm, and some type
    /// implementing the `Linkage` trait, representing the prologue/epilogue details.
    pub fn new(inner: S, ranges: Vec<core::ops::Range<u16>>) -> Self {
        Self { inner, ranges }
    }
}

impl<S, I> SearchAlgorithm for MemoryAccessSearch<S, I>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
{
    type Item = I;
    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        loop {
            let candidate = self.inner.generate()?;

            return Some(candidate);
        }
    }
}

/// A static analysis pass for ensuring that programs do not access memory outside of the allow
/// ranges.
pub struct SearchTrace<'a, S: SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
    func: &'a dyn Fn(&Candidate<I>),
}

impl<'a, S: SearchAlgorithm<Item = I>, I: Instruction> SearchTrace<'a, S, I> {
    /// Constructs a new LinkageSearch object, from an inner search algorithm, and some type
    /// implementing the `Linkage` trait, representing the prologue/epilogue details.
    pub fn new<'b>(inner: S, func: &'b dyn Fn(&Candidate<I>)) -> SearchTrace<'b, S, I> 
        where 'a: 'b, 'b: 'a
    {
        Self { inner, func }
    }
}

impl<S, I> SearchAlgorithm for SearchTrace<'_, S, I>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
{
    type Item = I;
    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
            let candidate = self.inner.generate()?;
            (self.func)(&candidate);

            return Some(candidate);
    }
}
