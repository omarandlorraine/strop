//! The `armv4t` module, the strop backend for targeting the ARMv4T CPUs, including the ARM7TDMI.
mod diss;
pub mod isa;

pub use isa::Insn;
