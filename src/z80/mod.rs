//! A back-end targeting the Z80, a well-known 8-bit retro CPU.
mod diss;
mod emu;
mod isa;
mod sdcccall1;

pub use emu::Emulator;
pub use isa::Insn;
pub use sdcccall1::SdccCall1;

/// Returns an empty `__sdcccall(1)` function
pub fn sdcccall1<Params, RetVal>() -> SdccCall1<Params, RetVal> {
    use crate::Step;
    SdccCall1::first()
}
