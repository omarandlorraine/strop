//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::BruteForceSearch;

/// Returns a default `ThumbInstructionSet`
pub fn thumb() -> instruction_set::ThumbInstructionSet {
    instruction_set::ThumbInstructionSet::default()
}

impl BruteForceSearch<crate::armv4t::instruction_set::ThumbInstructionSet> {
    pub fn aapcs32(self, func: fn(i32, i32) -> Option<i32>) -> testers::Aapcs32 {
        testers::Aapcs32::new(self, func)
    }
}
