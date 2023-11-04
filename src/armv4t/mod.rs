//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;

/// Returns a default `ThumbInstructionSet`
pub fn thumb() -> instruction_set::ThumbInstructionSet {
    instruction_set::ThumbInstructionSet::default()
}
