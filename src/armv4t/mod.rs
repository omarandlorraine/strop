//! The `armv4t` module, the strop back-end for targeting the ARMv4T CPUs, including the ARM7TDMI.
pub mod aapcs32;
mod diss;
mod subroutine;
pub mod isa;
mod emu;

pub use isa::Insn;
pub use subroutine::Subroutine;
pub use emu::Emulator;
