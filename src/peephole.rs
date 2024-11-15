//! This module implements a constraint which narrows the search space by ensuring that any
//! sequence caught by a peephole optimization is skipped. This is especially useful for the
//! BruteForce search algorithm.

use crate::Constrain;
use crate::Iterable;
use crate::Peephole;
use crate::Sequence;

/// A constraint for checking that a code sequence does not contain any two consecutive
/// instructions which a peephole optimizer would catch.
///
/// The `report` method will point out where such a code sequence would be replaced.
#[derive(Debug)]
pub struct PeepholeOptimizer<'a, Insn>
where
    Insn: Peephole,
{
    seq: &'a mut Sequence<Insn>,
}

impl<Insn> PeepholeOptimizer<'_, Insn>
where
    Insn: Peephole,
{
    /// builds a new `NotLive` struct.
    pub fn new<'a>(seq: &'a mut Sequence<Insn>) -> PeepholeOptimizer<'a, Insn> {
        PeepholeOptimizer::<'a, Insn> { seq }
    }

    fn check(&self, offset: usize) -> bool {
        if offset >= self.seq.len() - 1 {
            false
        } else {
            Insn::check(&self.seq[offset], &self.seq[offset + 1])
        }
    }
}

impl<Insn> Constrain<Insn> for PeepholeOptimizer<'_, Insn>
where
    Insn: Peephole + Iterable,
{
    fn fixup(&mut self) {
        for offset in 0..(self.seq.len() - 1) {
            while self.check(offset) {
                self.seq.mut_at(Insn::modify, offset);
            }
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        if self.check(offset) {
            vec!["The next two instructions are peephole optimizable".to_string()]
        } else {
            vec![]
        }
    }
}
