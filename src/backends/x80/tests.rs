use crate::backends::x80::X80;
use crate::backends::x80::data::InstructionData;

// The instruction set includes loads of useless instructions like `ld b, b`, which loads a
// register with itself. Having no effect on flags or anything. These instructions are NOPs, but
// not the canonical NOP.
fn not_a_useless_move(data: &InstructionData) {
    if data.mnemonic == "ld" {
        assert_ne!(data.operands[0], data.operands[1], "{:?}", data);
    }
}

// checks if the flow control bit is set correctly in the instruction data thing. basically it has
// to agree with what the opcode says it is.
fn flow_control(data: &InstructionData) {
    assert_eq!(
        data.flow_control,
        matches!(
            data.mnemonic,
            "jp" | "jr" | "ret" | "reti" | "retn" | "call" | "rst" | "djnz"
        ),
        "{:?}",
        data
    )
}

pub(crate) fn std_x80_tests<I: X80>() {
    let mut i = I::first();

    while i.increment().is_ok() {
        let data = i.decode();
        not_a_useless_move(data);
        flow_control(data);
    }
}
