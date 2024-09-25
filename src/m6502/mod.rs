//! This is the back-end targeting the MOS 6502, a well-known 8-bit retro CPU.

mod csp;
mod diss;
mod isa;
mod subroutine;

pub use subroutine::Subroutine;

pub use csp::Prune;
pub use isa::Insn;

pub use mos6502::instruction::Cmos6502;
pub use mos6502::instruction::Nmos6502;
pub use mos6502::instruction::RevisionA;
pub use mos6502::instruction::Ricoh2a03;
