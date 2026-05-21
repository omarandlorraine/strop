//!
//!

use crate::Instruction;
use crate::Sequence;
use crate::StaticAnalysis;

/// A struct, having a sequence of instructions, and a list of static analysis passes, as members.
#[derive(Clone)]
pub struct ExplorationPoint<I: Instruction> {
    /// Sequence of instructions
    sequence: Sequence<I>,

    /// Static analysis functions for ensuring correctness
    correctness: Vec<fn(&Sequence<I>) -> StaticAnalysis<I>>,
}

impl<I: Instruction> std::fmt::Debug for ExplorationPoint<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.sequence)
    }
}

impl<I: Instruction> ExplorationPoint<I> {
    /// Constructs a `ExplorationPoint<I>`.
    pub fn new() -> Self {
        Self {
            sequence: Sequence::<I>::default(),
            correctness: vec![],
        }
    }
    /// Returns the instruction sequence as bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.sequence.to_bytes()
    }
    /// Random mutation of the instruction sequence.
    pub fn mutate(&mut self) {
        self.sequence.mutate()
    }
    /// applies all fixups for an exhaustive search
    fn exhvalidate(&mut self) -> bool {
        let mut found_fixup = false;
        while let Err(fixup) = self.exhsa() {
            self.sequence.apply(&fixup);
            found_fixup = true;
        }
        found_fixup
    }

    /// Increments the instruction sequence, until all the static analysis passes pass.
    pub fn increment(&mut self) {
        if self.exhvalidate() {
            return;
        }
        self.sequence.increment();
        self.exhvalidate();
    }

    /// Adds a list of static analysis passes to the search, for ensuring correctness
    pub fn correctness(&mut self, sa: &[fn(&Sequence<I>) -> StaticAnalysis<I>]) -> &mut Self {
        self.correctness.extend(sa);
        self
    }

    /// Adds one static analysis pass
    pub fn add(&mut self, sa: fn(&Sequence<I>) -> StaticAnalysis<I>) -> &mut Self {
        self.correctness.push(sa);
        self
    }

    /// Of all the static analysis passes that fire, picks a random one and returns it.
    fn pick_static_analysis(&self) -> StaticAnalysis<I> {
        use rand::prelude::IndexedRandom;

        // There's no need to spend loads of compute time on the analysis; the sequence is likely
        // going to change soon anyway
        let fixups: Vec<_> = self
            .correctness
            .iter()
            .map(|s| s(&self.sequence))
            .filter(|s| s.is_err())
            .take(4)
            .collect();
        let mut rng = rand::rng();
        fixups.choose(&mut rng).unwrap_or(&Ok(())).clone()
    }

    /// Returns a `StaticAnalysis<I>` for either correctness or search culling
    fn exhsa(&self) -> StaticAnalysis<I> {
        self.pick_static_analysis()?;
        I::cull(&self.sequence)?;
        I::peephole(&self.sequence)
    }
}
