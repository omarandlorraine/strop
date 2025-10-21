use crate::Instruction as _;
use crate::backends::mips;
use mips::Instruction;

#[ignore]
#[test]
fn disassemblies_unique() {
    crate::generic_unit_tests::disassemblies_unique(Instruction::from_bytes(&[0, 0, 0, 0]), None);
}
