//! A strop backend targetting the Intel 8080.

mod instruction_set;
pub use instruction_set::Instruction;
pub(crate) mod data;
mod emu;
pub use emu::Emulator;

#[cfg(test)]
mod tests {
    use super::Instruction;
    use crate::Instruction as _;

    #[test]
    fn unique_disassembly() {
        crate::generic_unit_tests::disassemblies_unique(Instruction::first(), None);
    }
}
