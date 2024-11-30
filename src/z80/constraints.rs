use crate::z80::Insn;
use crate::Constrain;
use crate::Sequence;

/// A struct that defines which constraints to apply to a search
#[derive(Clone, Copy, Default, Debug)]
pub struct Constraints {
    bb: bool,
    leaffn: bool,
    purefn: bool,
}

impl Constraints {
    /// the purpose of this is evident
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds basic block to the Constraint, so that the search considers only those code sequences
    /// which are basic blocks.
    pub fn basic_block(&mut self) -> Self {
        self.bb = true;
        *self
    }

    /// Search only for leaf functions
    pub fn leaf_function(&mut self) -> Self {
        self.leaffn = true;
        *self
    }

    /// Search only for pure functions
    pub fn pure_function(&mut self) -> Self {
        self.purefn = true;
        *self
    }
}

impl Constrain<Insn> for Constraints {
    fn fixup(&self, _candidate: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        None
    }
}
