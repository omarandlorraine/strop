//! The `armv4t` module, the strop back-end for targeting the ARMv4T CPUs, including the ARM7TDMI.
mod diss;
mod emu;
mod function;
pub mod isa;
mod subroutine;

pub use emu::Emulator;
pub use isa::Insn;
pub use subroutine::Subroutine;
