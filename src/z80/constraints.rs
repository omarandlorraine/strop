use crate::z80::Insn;
use crate::Constrain;
use crate::Sequence;

use crate::z80::sdcccall1::SdccCall1ParameterList;

/// A struct that defines which constraints to apply to a search
#[derive(Clone, Copy, Default, Debug)]
pub struct Constraints {
    bb: bool,
    nopeep: bool,
    leaffn: bool,
    purefn: bool,
    nodata: bool,
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

    /// Removes peephole optimization from the search constraint. It's unclear why you'd want to do
    /// that.
    pub fn disable_peephole_optimization(&mut self) -> Self {
        self.nopeep = true;
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

impl Constrain<Sequence<Insn>> for Constraints {
    fn fixup(&self, sequence: &mut Sequence<Insn>) {
        // In a basic block, only the last instruction may do any flow control
        if self.bb {
            for i in 0..(sequence.len() - 1) {
                if sequence[i].is_flow_control() {
                    // We're looking for basic blocks, so this instruction may not do any flow control!
                    sequence[i].next_opcode();
                    continue;
                }
            }
        }

        if self.purefn {
            for i in 0..sequence.len() {
                while !sequence[i].allowed_in_pure_functions() {
                    // We're looking for pure functions; this should not read or write anything!
                    sequence[i].next_opcode();
                    continue;
                }
            }
        }
    }
}

impl<Params: SdccCall1ParameterList, ReturnValue>
    Constrain<crate::z80::SdccCall1<Params, ReturnValue>> for Constraints
{
    fn fixup(&self, function: &mut crate::z80::SdccCall1<Params, ReturnValue>) {
        if !self.nodata {
            function.dataflow_analysis();
        }
        <Constraints as Constrain<Sequence<Insn>>>::fixup(self, function);
    }
}
