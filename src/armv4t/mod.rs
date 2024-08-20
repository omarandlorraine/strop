//! The `armv4t` module, the strop back-end for targeting the ARMv4T CPUs, including the ARM7TDMI.
mod diss;
pub mod isa;

pub use isa::Insn;
