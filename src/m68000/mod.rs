//! This is the strop backend, targetting the Motorola 68000
mod diss;
mod isa;
mod prune;

pub use isa::Insn;
pub use prune::Prune;
