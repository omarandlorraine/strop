//! This is the strop back-end targeting the Motorola 68000
mod diss;
mod isa;
mod subroutine;
mod emu;

pub use emu::Emulator;
pub use isa::Insn;
