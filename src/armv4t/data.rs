//! Dataflow analysis for ARM instructions.
//!
//! This module implements dataflow analysis for ARM instructions.
use crate::armv4t::Insn;
use crate::static_analysis::Fixup;

/// Condition flags. Conditional instructions read from this, and instructions which set the
/// condition flags (such as `ands`) write to the condition flags.
#[derive(Debug)]
pub struct ConditionFlags;

impl crate::dataflow::DataFlow<ConditionFlags> for Insn {
    fn reads(&self, _datum: &ConditionFlags) -> bool {
        // reading from the Condition Flags means, the instruction is conditional.
        self.0 < 0xe000_0000
    }

    fn writes(&self, _datum: &ConditionFlags) -> bool {
        self.decode().updates_condition_flags()
    }

    fn sa(&self, offset: usize) -> Fixup<Self> {
        Fixup::new("ConditionDataflow", Self::next_opcode, offset)
    }
}
