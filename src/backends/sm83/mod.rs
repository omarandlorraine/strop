//! A strop backend targetting the SM83, also known as the LR35902 or the Gameboy CPU.
//!
//! It's a bit like a Z80 but not quite.

mod instruction_set;
pub use instruction_set::Instruction;
mod data;
mod emu;
pub use emu::Emu as Emulator;
mod sdcccall;

#[cfg(test)]
mod tests {
    use super::Instruction;
    use crate::Instruction as _;

    #[test]
    fn std_x80_tests() {
        crate::backends::x80::tests::std_x80_tests::<Instruction>();
    }

    #[test]
    fn unique_disassembly() {
        crate::generic_unit_tests::disassemblies_unique(Instruction::first(), None);
    }
}
