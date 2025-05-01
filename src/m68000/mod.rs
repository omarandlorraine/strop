//! This is the strop back-end targeting the Motorola 68000
mod diss;
mod emu;
mod isa;
mod subroutine;

pub use emu::Emulator;
pub use isa::Insn;
