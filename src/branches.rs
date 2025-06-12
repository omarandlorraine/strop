//! Module defining traits and things for scrutinizing and treating instructions which implement
//! relative jumps, optional or otherwise. This includes such instructions as:
//!
//!  - branches (these are typically conditional branches, such as the `beq` instruction on the
//!    6502, but are not necessarily conditional)
//!
//!  - relative jumps like the 6809 and PDP-11 have.
//!
//!  - skip instructions à la PDP-8 (these are the same as a branch with an implicit forward
//!    destination)
//!
//! Strop will use these traits to ensure that, for example, a subroutine does not contain an
//! instruction that jumps outside of the subroutine.

use crate::StaticAnalysis;

/// Implement this trait on an instruction type if that instruction set has relative
/// jumps/branches/skips
pub trait Branch: Sized {
    /// Returns the branch's target relative to the instruction's address
    fn offset(&self) -> Option<isize> {
        None
    }

    /// Returns a StaticAnalysis if the forward branch is out of bounds
    fn branch_fixup(&self, _permissibles: &[isize]) -> StaticAnalysis<Self> {
        Ok(())
    }
}
