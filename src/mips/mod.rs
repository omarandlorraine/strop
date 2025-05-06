//! A back-end targeting MIPS processors.

pub mod emu;
mod isa;
mod o32;
mod subroutine;

pub use isa::Insn;
pub use o32::O32;
pub use subroutine::Subroutine;
