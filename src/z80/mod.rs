//! A back-end targeting the Z80, a well-known 8-bit retro CPU.
mod diss;
mod emu;
mod isa;
mod sdcccall1;

pub use emu::Emulator;
pub use isa::Insn;
pub use sdcccall1::SdccCall1;

/// Returns an empty Z80 subroutine.
pub fn subroutine() -> crate::Subroutine<crate::Sequence<Insn>> {
    use crate::Step;
    use crate::subroutine::AsSubroutine;

    crate::Sequence::<Insn>::first().as_subroutine()
}
