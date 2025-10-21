//! A backend targetting the ARMv4T architecture
//!
//! Coprocessors not supported; this will use softfloats

mod instruction_set;
pub use instruction_set::Instruction;

mod aapcs32;
pub use aapcs32::Aapcs32;

mod dataflow;

#[cfg(test)]
mod test;
