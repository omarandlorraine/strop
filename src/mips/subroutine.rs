use crate::mips::Insn;

/// Represents a MIPS subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;
