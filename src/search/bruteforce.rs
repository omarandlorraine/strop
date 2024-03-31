//! Module containing definitions of miscellaneous search strategies.

use crate::Compatibility;
use crate::Fitness;
use crate::Linkage;
use crate::Peephole;
use crate::SearchAlgorithm;
use crate::{Candidate, Instruction};

/// Iterates across the entire search space, shortest programs first.
#[derive(Debug)]
pub struct BruteForceSearch<I: Instruction + PartialOrd + PartialEq> {
    curr: Vec<I>,
}

impl<I: Instruction + std::cmp::PartialOrd> SearchAlgorithm for BruteForceSearch<I> {
    type Item = I;
    fn score(&mut self, _: f32) {}

    fn fitness(&mut self, _cand: &Candidate<Self::Item>) -> Fitness {
        Fitness::Passes(0.0)
    }

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

impl<S: SearchAlgorithm<Item = I>, I: PartialOrd + Instruction> BruteForce<I>
    for LengthLimitedSearch<S, I>
{
}

impl<S: SearchAlgorithm<Item = I>, I: PartialOrd + Instruction> BruteForce<I>
    for PeepholeOptimizedBruteForceSearch<S, I>
{
}

impl<I: PartialOrd + Instruction> BruteForce<I>
    for BruteForceSearch<I>
{
}

pub trait BruteForce<I: Instruction + std::cmp::PartialOrd + std::cmp::PartialEq> {
    /// Limits the length of the generated programs
    fn limit_length(self, length: usize) -> LengthLimitedSearch<Self, I>
    where
        Self: SearchAlgorithm<Item = I> + Sized,
    {
        LengthLimitedSearch {
            inner: self,
            length,
        }
    }

    /// Adds peephole optimization; culls the search space
    fn peephole(self) -> PeepholeOptimizedBruteForceSearch<Self, I>
    where
        I: Peephole,
        Self: SearchAlgorithm<Item = I> + Sized,
    {
        PeepholeOptimizedBruteForceSearch { inner: self }
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
}

#[derive(Debug)]
/// A SearchAlgorithm that rejects any programs having a longer length than the one specified.
pub struct LengthLimitedSearch<S: SearchAlgorithm<Item = I>, I: Instruction> {
    inner: S,
    length: usize,
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> SearchAlgorithm for LengthLimitedSearch<S, I> {
    type Item = I;

    fn fitness(&mut self, cand: &Candidate<Self::Item>) -> Fitness {
        self.inner.fitness(cand)
    }

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

impl<I: Instruction + PartialOrd + PartialEq> Default for BruteForceSearch<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// A static analysis pass which selects instructions for compatibility with particular CPU
/// variants.
#[derive(Debug)]
pub struct PeepholeOptimizedBruteForceSearch<S: SearchAlgorithm<Item = I>, I: Instruction>
where
    I: PartialEq,
{
    inner: S,
}

impl<S, I> SearchAlgorithm for PeepholeOptimizedBruteForceSearch<S, I>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction + PartialEq + Peephole,
{
    type Item = I;

    fn fitness(&mut self, candidate: &Candidate<I>) -> Fitness {
        self.inner.fitness(candidate)
    }

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        use crate::SearchCull;

        loop {
            let candidate = self.inner.generate()?;
            let (offs, cull) = I::peephole(&candidate);
            match cull {
                SearchCull::Okay => return Some(candidate),
                SearchCull::SkipTo(something) => self.replace(offs, something),
            }
        }
    }
}

/// A static analysis pass which selects instructions for compatibility with particular CPU
/// variants.
#[derive(Debug)]
pub struct CompatibilitySearch<S: SearchAlgorithm<Item = I>, I: Instruction, C: Compatibility<I>>
where
    I: PartialEq,
{
    compatibility: C,
    inner: S,
}

impl<S, I, C> CompatibilitySearch<S, I, C>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction + PartialEq,
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
    I: Instruction + PartialEq,
    C: Compatibility<I>,
{
    type Item = I;

    fn fitness(&mut self, candidate: &Candidate<I>) -> Fitness {
        use crate::SearchCull;
        if candidate
            .instructions
            .iter()
            .map(|i| self.compatibility.check(i))
            .any(|c| c != SearchCull::Okay)
        {
            Fitness::FailsStaticAnalysis
        } else {
            self.inner.fitness(candidate)
        }
    }

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
    linkage: L,
    inner: S,
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

    fn fitness(&mut self, candidate: &Candidate<I>) -> Fitness {
        if self.linkage.check(candidate) {
            Fitness::FailsStaticAnalysis
        } else {
            self.inner.fitness(candidate)
        }
    }

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        loop {
            let candidate = self.inner.generate()?;
            if self.linkage.fixup(&mut self.inner, &candidate) {
                return Some(candidate);
            }
        }
    }
}

/// A static analysis pass for ensuring that programs do not access memory outside of the allow
/// ranges.
#[derive(Debug)]
pub struct SearchTrace<S: SearchAlgorithm<Item = I>, I: Instruction> {
    func: fn(&Candidate<I>),
    inner: S,
}

impl<S: SearchAlgorithm<Item = I>, I: Instruction> SearchTrace<S, I> {
    /// Constructs a new LinkageSearch object, from an inner search algorithm, and some type
    /// implementing the `Linkage` trait, representing the prologue/epilogue details.
    pub fn new(inner: S, func: fn(&Candidate<I>)) -> SearchTrace<S, I> {
        Self { inner, func }
    }
}

impl<S, I> SearchAlgorithm for SearchTrace<S, I>
where
    S: Sized + SearchAlgorithm<Item = I>,
    I: Instruction,
{
    type Item = I;

    fn fitness(&mut self, candidate: &Candidate<I>) -> Fitness {
        self.inner.fitness(candidate)
    }

    fn score(&mut self, score: f32) {
        self.inner.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<I>) {
        self.inner.replace(offset, instruction)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        let candidate = self.inner.generate()?;
        (self.func)(&candidate);
        Some(candidate)
    }
}
