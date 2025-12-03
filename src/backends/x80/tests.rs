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
fn flow_control<I: X80>(insn: &I) {
    let data = insn.decode();
    assert_eq!(
        data.flow_control,
        matches!(
            data.mnemonic,
            "jp" | "jr" | "ret" | "reti" | "retn" | "call" | "rst" | "djnz"
        ),
        "{insn:?}",
    )
}

fn length<I: X80>(insn: &I) {
    use crate::backends::x80::EmuInterface;

    let decoded = insn.decode();

    // compare the length in the struct with the length of the encoding
    let bytes = insn.to_bytes();
    assert_eq!(bytes.len(), decoded.bytes, "{decoded:?}");

    // Also check the emulator agrees with the instruction length. But we can't do that with
    // flow control instructions, they can do anything with the program counter.
    if decoded.flow_control {
        return;
    }

    // Also the `stop` instruction is a weirdo: although it is a single byte, the CPU can skip the
    // next byte in the instruction sequence. This is a hardware bug that's faithfully emulated by
    // `mizu_core`.
    if decoded.mnemonic == "stop" {
        return;
    }

    // Also the "repeat" instructions can't be used this way
    if matches!(
        decoded.mnemonic,
        "ldir" | "cpir" | "inir" | "otir" | "lddr" | "cpdr" | "indr" | "otdr"
    ) {
        return;
    }

    // Check the emulator agrees with the instruction length
    let mut emu = I::Emulator::default();
    for (addr, byte) in bytes.iter().enumerate() {
        emu.poke(addr.try_into().unwrap(), *byte);
    }
    emu.single_step().unwrap();
    assert_eq!(emu.get_pc(), decoded.bytes as u16, "{insn:?}");
}

pub(crate) fn std_x80_tests<I: X80>() {
    let mut i = I::first();

    while i.increment().is_ok() {
        println!("{i:?}");
        let data = i.decode();
        not_a_useless_move(data);
        flow_control(&i);
        length(&i);
    }
}
