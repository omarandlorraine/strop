//! The `armv4t` module, the strop back-end for targeting the ARMv4T CPUs, including the ARM7TDMI.
pub mod aapcs32;
mod diss;
mod emu;
pub mod isa;

pub use emu::Emulator;
pub use isa::Insn;
