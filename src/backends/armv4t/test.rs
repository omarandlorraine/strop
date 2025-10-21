use crate::Callable;
use crate::Sequence;
use crate::backends::armv4t;
use armv4t::Aapcs32;
use armv4t::Instruction;

#[test]
fn call_identity_function() {
    let function = Aapcs32::<u32, u32>::from(Sequence::<Instruction>::from(vec![
        Instruction(0x00000050),
        Instruction(0xe12fff1e),
    ]));

    for v in [1, 2, 3, 0, 0xaaaaaaaa, 0x80000000] {
        assert_eq!(function.call(v), Ok(v));
    }
}

#[ignore]
#[test]
fn disassemblies_unique() {
    crate::generic_unit_tests::disassemblies_unique::<Instruction>(Instruction(0xe000_0000), None);
}

#[ignore]
#[test]
fn deduped() {
    use crate::generic_unit_tests::list_all_encodings;

    list_all_encodings::<Instruction>("and r0, r0, r0", Instruction(0xe000_0000), None);
}
