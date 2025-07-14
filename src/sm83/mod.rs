//! A back-end supporting the SM83 (which is also known by other names, including LR35902, or the
//! Gameboy CPU)

mod data;
mod emu;
pub mod isa;

pub use isa::Insn;

/// An SM83 subroutine adhering to the SDCCCALL1 calling convention
pub type SdccCall1<Params, RetVal> = crate::x80::sdcccall1::SdccCall1<Insn, Params, RetVal>;

#[cfg(test)]
mod test {
    #[test]
    fn opcodes() {
        use super::data::CBPREFIXED;
        use super::data::UNPREFIXED;
        for (opcode, idata) in UNPREFIXED.iter().enumerate() {
            if let Some(idata) = idata {
                assert_eq!(opcode as u8, idata.opcode);
            }
        }
        for (opcode, idata) in CBPREFIXED.iter().enumerate() {
            if let Some(idata) = idata {
                assert_eq!(opcode as u8, idata.opcode);
            }
        }
    }
}
