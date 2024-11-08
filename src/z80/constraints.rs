use crate::z80::Insn;
use crate::Constrain;
use crate::Sequence;

struct BasicBlock;

impl BasicBlock {
    fn check(seq: &Sequence<Insn>, offset: usize) -> bool {
        if offset == seq.len() - 1 {
            false
        } else {
            seq[offset].is_flow_control()
        }
    }
}

impl Constrain<Insn> for BasicBlock {
    fn fixup(&self, seq: &mut Sequence<Insn>, offset: usize) {
        if Self::check(seq, offset) {
            seq.mut_at(Insn::next_opcode, offset);
        }
    }
    fn report(&self, seq: &Sequence<Insn>, offset: usize) -> Vec<String> {
        if Self::check(seq, offset) {
            vec![
                "This flow control instruction is not allowed at this point in a basic block"
                    .to_string(),
            ]
        } else {
            vec![]
        }
    }
}

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
    fn fixup(&self, sequence: &mut Sequence<Insn>, offset: usize) {
        if self.bb {
            BasicBlock.fixup(sequence, offset)
        }
    }

    fn report(&self, sequence: &Sequence<Insn>, offset: usize) -> Vec<String> {
        let mut reports = vec![];
        if self.bb {
            reports.extend(BasicBlock.report(sequence, offset))
        }
        reports
    }
}
