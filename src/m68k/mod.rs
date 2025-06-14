//! A backend for generating code that will run on the Motorola 68000
mod diss;
mod emu;
mod isa;
mod regparm;

pub use isa::Insn;
pub use regparm::Regparm;
