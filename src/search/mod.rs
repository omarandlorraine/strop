//! Module containing miscellaneous search algorithms that are generic across instruction sets.
//! Also contains some static analysis passes.
mod stochastic;
mod bruteforce;

pub use stochastic::StochasticSearch;
pub use bruteforce::BruteForceSearch;
use crate::{Instruction, Fixup, SearchAlgorithm, Candidate, Fitness};

/// A convenience for calling a function with every putative program
#[derive(Debug)]
pub struct SearchTrace<S: crate::SearchAlgorithm<Item = I>, I: crate::Instruction> {
    func: fn(&crate::Candidate<I>),
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

    fn replace<F: Fixup<I>>(&mut self, offset: usize, fixup: F) {
        self.inner.replace(offset, fixup)
    }

    fn generate(&mut self) -> Option<Candidate<I>> {
        let candidate = self.inner.generate()?;
        (self.func)(&candidate);
        Some(candidate)
    }
}
