//! Module containing conveniences for static analysis. The definition of `StaticAnalysis` means we
//! can use the `?` operator for flow control. This makes it reasonable to check for logic errors,
//! culling the search space.

use crate::IterationResult;

/// The result of static analysis is either "this is okay", or "there's a problem, but here's how
/// to fix it". For this reason I've defined `StaticAnalysis` to be a `Result` type. The `Err`
/// variant has a payload called `Fixup`, which of course carries information about how to fix
/// something up.
pub type StaticAnalysis<Instruction> = Result<(), Fixup<Instruction>>;

/// An erroneous result of static analysis. Explains why a code sequence has been found to be illogical
/// or unsuitable, and provides a way to prune such a sequence from the search.
#[derive(Debug)]
pub struct Fixup<Instruction: ?Sized> {
    /// Specifies at what offset into this sequence the problem was found
    pub offset: usize,
    /// How to fix the problem
    pub advance: fn(&mut Instruction) -> IterationResult,
    /// Human-readable description of the problem
    pub reason: &'static str,
}

impl<Instruction> Fixup<Instruction> {
    /// Construct a new Fixup
    pub fn new(
        reason: &'static str,
        advance: fn(&mut Instruction) -> IterationResult,
        offset: usize,
    ) -> Fixup<Instruction> {
        Fixup {
            offset,
            advance,
            reason,
        }
    }
    /// Constructs an Err(self)
    pub fn err(
        reason: &'static str,
        advance: fn(&mut Instruction) -> IterationResult,
        offset: usize,
    ) -> StaticAnalysis<Instruction> {
        Err(Self::new(reason, advance, offset))
    }

    /// Returns either an Ok or a Err, depending on the value of cond.
    pub fn check(
        cond: bool,
        reason: &'static str,
        advance: fn(&mut Instruction) -> IterationResult,
        offset: usize,
    ) -> StaticAnalysis<Instruction> {
        if cond {
            Ok(())
        } else {
            Self::err(reason, advance, offset)
        }
    }
}
