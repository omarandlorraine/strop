//! A back-end targeting the Z80, a well-known 8-bit retro CPU.
mod diss;
mod emu;
mod isa;
mod sdcccall1;
mod subroutine;

pub use emu::Emulator;
pub use isa::Insn;
pub use sdcccall1::SdccCall1;
pub use subroutine::Subroutine;

/// Returns an empty `__sdcccall(1)` function
pub fn sdcccall1() -> SdccCall1 {
    use crate::Step;
    SdccCall1::first()
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn bruteforce_find_zero() {
        fn z(_s: u8) -> crate::RunResult<u8> {
            Ok(b'0')
        }

        use crate::AsBruteforce;
        crate::z80::SdccCall1::default()
            .bruteforce(z as fn(u8) -> crate::RunResult<u8>)
            .search()
            .unwrap();
    }
}
