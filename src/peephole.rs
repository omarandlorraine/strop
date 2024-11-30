//! This module implements a constraint which narrows the search space by ensuring that any
//! sequence caught by a peephole optimization is skipped. This is especially useful for the
//! BruteForce search algorithm.

use crate::Constrain;
use crate::Iterable;
use crate::Sequence;

pub trait Peephole {
    //! A trait for very local peephole optimizations. It's generic across `T`, a type intended to
    //! represent machine instructions.
    //!
    //! The default implementation is effectively a no-op.

    /// Modifies the instruction
    fn modify(&mut self) -> bool {
        unreachable!();
    }

    /// Checks if two instructions may not follow eachother.
    fn check(_a: &Self, _b: &Self) -> bool {
        false
    }
}

/// A constraint for checking that a code sequence does not contain any two consecutive
/// instructions which a peephole optimizer would catch.
///
/// The `report` method will point out where such a code sequence would be replaced.
#[derive(Debug, Default)]
pub struct PeepholeOptimizer<Insn>
where
    Insn: Peephole,
{
    i: std::marker::PhantomData<Insn>,
}

impl<Insn> PeepholeOptimizer<Insn>
where
    Insn: Peephole,
{
    fn check(&self, seq: &Sequence<Insn>, offset: usize) -> bool {
        if offset >= seq.len() - 1 {
            false
        } else {
            Insn::check(&seq[offset], &seq[offset + 1])
        }
    }
}

impl<Insn> Constrain<Insn> for PeepholeOptimizer<Insn>
where
    Insn: Peephole + Iterable,
{
    fn fixup(&self, seq: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        for offset in 0..(seq.len() - 1) {
            if self.check(seq, offset) {
                seq.mut_at(Insn::modify, offset);
                return Some((offset, "peephole"));
            }
        }
        None
    }
}
