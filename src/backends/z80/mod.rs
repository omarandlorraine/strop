//! A strop backend targetting the Zilog Z80.

mod instruction_set;
pub use instruction_set::Instruction;
pub(crate) mod data;
mod emu;
pub use emu::Emulator;
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
