use crate::backends::mos6502::Instruction;

fn generic_test<V: mos6502::Variant>() {
    use crate::Instruction as _;
    crate::generic_unit_tests::sanity_checks::<Instruction<V>>();
    crate::generic_unit_tests::disassemblies_unique::<Instruction<V>>(
        crate::Instruction::increment,
    );
}

#[test]
fn cmos() {
    generic_test::<mos6502::instruction::Cmos6502>();
}

#[test]
fn nmos() {
    generic_test::<mos6502::instruction::Nmos6502>();
}

#[test]
fn r2a03() {
    generic_test::<mos6502::instruction::Ricoh2a03>();
}

#[test]
fn ra() {
    generic_test::<mos6502::instruction::RevisionA>();
}
