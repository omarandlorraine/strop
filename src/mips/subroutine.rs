use crate::mips::Insn;

/// Represents a MIPS subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;

impl Default for Subroutine {
    fn default() -> Self {
        use crate::subroutine::ToSubroutine;
        use crate::Step;
        crate::Sequence::<Insn>::first().to_subroutine()
    }
}
