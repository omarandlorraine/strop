#[cfg(test)]
mod test {
    #[test]
    fn opcodes() {
        use crate::Instruction as _;
        use crate::backends::x80::X80;
        use crate::backends::z80::Instruction;
        for opcode in 0..=255 {
            if !matches!(opcode, 0xcb | 0xed) {
                if let Some(insn) = Instruction::from_bytes(&[opcode, 0, 0, 0, 0]) {
                    assert_eq!(insn.decode().opcode, opcode, "{:?}", insn);
                }
            }
            if let Some(insn) = Instruction::from_bytes(&[0xed, opcode, 0, 0, 0]) {
                assert_eq!(insn.decode().opcode, opcode, "{:?}", insn);
            }
        }
    }

    #[test]
    fn dasm() {
        use crate::Instruction as _;
        use crate::backends::z80::Instruction;
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x00]).unwrap()),
            "nop"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x01, 0x34, 0x12]).unwrap()),
            "ld bc, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x02]).unwrap()),
            "ld (bc), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x03]).unwrap()),
            "inc bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x04]).unwrap()),
            "inc b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x05]).unwrap()),
            "dec b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x06, 0x45]).unwrap()),
            "ld b, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x07]).unwrap()),
            "rlca"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x08]).unwrap()),
            "ex af, af'"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x09]).unwrap()),
            "add hl, bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0A]).unwrap()),
            "ld a, (bc)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0B]).unwrap()),
            "dec bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0C]).unwrap()),
            "inc c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0D]).unwrap()),
            "dec c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0E, 0x45]).unwrap()),
            "ld c, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x0F]).unwrap()),
            "rrca"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x10, 0x45]).unwrap()),
            "djnz 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x11, 0x34, 0x12]).unwrap()),
            "ld de, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x12]).unwrap()),
            "ld (de), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x13]).unwrap()),
            "inc de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x14]).unwrap()),
            "inc d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x15]).unwrap()),
            "dec d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x16, 0x45]).unwrap()),
            "ld d, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x17]).unwrap()),
            "rla"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x18, 0x45]).unwrap()),
            "jr 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x19]).unwrap()),
            "add hl, de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1A]).unwrap()),
            "ld a, (de)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1B]).unwrap()),
            "dec de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1C]).unwrap()),
            "inc e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1D]).unwrap()),
            "dec e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1E, 0x45]).unwrap()),
            "ld e, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x1F]).unwrap()),
            "rra"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x20, 0x45]).unwrap()),
            "jr nz, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x21, 0x34, 0x12]).unwrap()),
            "ld hl, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x22, 0x34, 0x12]).unwrap()),
            "ld (1234h), hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x23]).unwrap()),
            "inc hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x24]).unwrap()),
            "inc h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x25]).unwrap()),
            "dec h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x26, 0x45]).unwrap()),
            "ld h, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x27]).unwrap()),
            "daa"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x28, 0x45]).unwrap()),
            "jr z, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x29]).unwrap()),
            "add hl, hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2A, 0x34, 0x12]).unwrap()),
            "ld hl, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2B]).unwrap()),
            "dec hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2C]).unwrap()),
            "inc l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2D]).unwrap()),
            "dec l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2E, 0x45]).unwrap()),
            "ld l, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x2F]).unwrap()),
            "cpl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x30, 0x45]).unwrap()),
            "jr nc, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x31, 0x34, 0x12]).unwrap()),
            "ld sp, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x32, 0x34, 0x12]).unwrap()),
            "ld (1234h), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x33]).unwrap()),
            "inc sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x34]).unwrap()),
            "inc (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x35]).unwrap()),
            "dec (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x36, 0x45]).unwrap()),
            "ld (hl), 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x37]).unwrap()),
            "scf"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x38, 0x45]).unwrap()),
            "jr c, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x39]).unwrap()),
            "add hl, sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3A, 0x34, 0x12]).unwrap()),
            "ld a, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3B]).unwrap()),
            "dec sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3C]).unwrap()),
            "inc a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3D]).unwrap()),
            "dec a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3E, 0x45]).unwrap()),
            "ld a, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x3F]).unwrap()),
            "ccf"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x41]).unwrap()),
            "ld b, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x42]).unwrap()),
            "ld b, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x43]).unwrap()),
            "ld b, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x44]).unwrap()),
            "ld b, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x45]).unwrap()),
            "ld b, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x46]).unwrap()),
            "ld b, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x47]).unwrap()),
            "ld b, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x48]).unwrap()),
            "ld c, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4A]).unwrap()),
            "ld c, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4B]).unwrap()),
            "ld c, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4C]).unwrap()),
            "ld c, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4D]).unwrap()),
            "ld c, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4E]).unwrap()),
            "ld c, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x4F]).unwrap()),
            "ld c, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x50]).unwrap()),
            "ld d, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x51]).unwrap()),
            "ld d, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x53]).unwrap()),
            "ld d, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x54]).unwrap()),
            "ld d, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x55]).unwrap()),
            "ld d, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x56]).unwrap()),
            "ld d, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x57]).unwrap()),
            "ld d, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x58]).unwrap()),
            "ld e, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x59]).unwrap()),
            "ld e, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x5A]).unwrap()),
            "ld e, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x5C]).unwrap()),
            "ld e, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x5D]).unwrap()),
            "ld e, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x5E]).unwrap()),
            "ld e, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x5F]).unwrap()),
            "ld e, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x60]).unwrap()),
            "ld h, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x61]).unwrap()),
            "ld h, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x62]).unwrap()),
            "ld h, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x63]).unwrap()),
            "ld h, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x65]).unwrap()),
            "ld h, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x66]).unwrap()),
            "ld h, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x67]).unwrap()),
            "ld h, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x68]).unwrap()),
            "ld l, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x69]).unwrap()),
            "ld l, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x6A]).unwrap()),
            "ld l, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x6B]).unwrap()),
            "ld l, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x6C]).unwrap()),
            "ld l, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x6E]).unwrap()),
            "ld l, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x6F]).unwrap()),
            "ld l, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x70]).unwrap()),
            "ld (hl), b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x71]).unwrap()),
            "ld (hl), c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x72]).unwrap()),
            "ld (hl), d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x73]).unwrap()),
            "ld (hl), e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x74]).unwrap()),
            "ld (hl), h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x75]).unwrap()),
            "ld (hl), l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x76]).unwrap()),
            "halt"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x77]).unwrap()),
            "ld (hl), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x78]).unwrap()),
            "ld a, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x79]).unwrap()),
            "ld a, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x7A]).unwrap()),
            "ld a, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x7B]).unwrap()),
            "ld a, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x7C]).unwrap()),
            "ld a, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x7D]).unwrap()),
            "ld a, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x7E]).unwrap()),
            "ld a, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x80]).unwrap()),
            "add a, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x81]).unwrap()),
            "add a, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x82]).unwrap()),
            "add a, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x83]).unwrap()),
            "add a, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x84]).unwrap()),
            "add a, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x85]).unwrap()),
            "add a, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x86]).unwrap()),
            "add a, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x87]).unwrap()),
            "add a, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x88]).unwrap()),
            "adc a, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x89]).unwrap()),
            "adc a, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8A]).unwrap()),
            "adc a, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8B]).unwrap()),
            "adc a, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8C]).unwrap()),
            "adc a, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8D]).unwrap()),
            "adc a, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8E]).unwrap()),
            "adc a, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x8F]).unwrap()),
            "adc a, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x90]).unwrap()),
            "sub a, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x91]).unwrap()),
            "sub a, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x92]).unwrap()),
            "sub a, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x93]).unwrap()),
            "sub a, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x94]).unwrap()),
            "sub a, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x95]).unwrap()),
            "sub a, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x96]).unwrap()),
            "sub a, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x97]).unwrap()),
            "sub a, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x98]).unwrap()),
            "sbc a, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x99]).unwrap()),
            "sbc a, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9A]).unwrap()),
            "sbc a, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9B]).unwrap()),
            "sbc a, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9C]).unwrap()),
            "sbc a, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9D]).unwrap()),
            "sbc a, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9E]).unwrap()),
            "sbc a, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0x9F]).unwrap()),
            "sbc a, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA0]).unwrap()),
            "and b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA1]).unwrap()),
            "and c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA2]).unwrap()),
            "and d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA3]).unwrap()),
            "and e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA4]).unwrap()),
            "and h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA5]).unwrap()),
            "and l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA6]).unwrap()),
            "and (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA7]).unwrap()),
            "and a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA8]).unwrap()),
            "xor b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xA9]).unwrap()),
            "xor c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAA]).unwrap()),
            "xor d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAB]).unwrap()),
            "xor e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAC]).unwrap()),
            "xor h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAD]).unwrap()),
            "xor l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAE]).unwrap()),
            "xor (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xAF]).unwrap()),
            "xor a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB0]).unwrap()),
            "or b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB1]).unwrap()),
            "or c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB2]).unwrap()),
            "or d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB3]).unwrap()),
            "or e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB4]).unwrap()),
            "or h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB5]).unwrap()),
            "or l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB6]).unwrap()),
            "or (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB7]).unwrap()),
            "or a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB8]).unwrap()),
            "cp b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xB9]).unwrap()),
            "cp c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBA]).unwrap()),
            "cp d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBB]).unwrap()),
            "cp e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBC]).unwrap()),
            "cp h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBD]).unwrap()),
            "cp l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBE]).unwrap()),
            "cp (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xBF]).unwrap()),
            "cp a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC0]).unwrap()),
            "ret nz"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC1]).unwrap()),
            "pop bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC2, 0x34, 0x12]).unwrap()),
            "jp nz, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC3, 0x34, 0x12]).unwrap()),
            "jp 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC4, 0x34, 0x12]).unwrap()),
            "call nz, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC5]).unwrap()),
            "push bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC6, 0x45]).unwrap()),
            "add a, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC7]).unwrap()),
            "rst $00"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC8]).unwrap()),
            "ret z"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xC9]).unwrap()),
            "ret"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xCA, 0x34, 0x12]).unwrap()),
            "jp z, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x00]).unwrap()),
            "rlc b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x01]).unwrap()),
            "rlc c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x02]).unwrap()),
            "rlc d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x03]).unwrap()),
            "rlc e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x04]).unwrap()),
            "rlc h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x05]).unwrap()),
            "rlc l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x06]).unwrap()),
            "rlc (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x07]).unwrap()),
            "rlc a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x08]).unwrap()),
            "rrc b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x09]).unwrap()),
            "rrc c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x0A]).unwrap()),
            "rrc d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x0B]).unwrap()),
            "rrc e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x0E]).unwrap()),
            "rrc (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x0F]).unwrap()),
            "rrc a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x10]).unwrap()),
            "rl b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x11]).unwrap()),
            "rl c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x12]).unwrap()),
            "rl d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x13]).unwrap()),
            "rl e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x14]).unwrap()),
            "rl h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x15]).unwrap()),
            "rl l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x16]).unwrap()),
            "rl (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x17]).unwrap()),
            "rl a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x18]).unwrap()),
            "rr b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x19]).unwrap()),
            "rr c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1A]).unwrap()),
            "rr d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1B]).unwrap()),
            "rr e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1C]).unwrap()),
            "rr h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1D]).unwrap()),
            "rr l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1E]).unwrap()),
            "rr (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x1F]).unwrap()),
            "rr a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x20]).unwrap()),
            "sla b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x21]).unwrap()),
            "sla c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x22]).unwrap()),
            "sla d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x23]).unwrap()),
            "sla e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x24]).unwrap()),
            "sla h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x25]).unwrap()),
            "sla l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x26]).unwrap()),
            "sla (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x27]).unwrap()),
            "sla a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x28]).unwrap()),
            "sra b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x29]).unwrap()),
            "sra c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2A]).unwrap()),
            "sra d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2B]).unwrap()),
            "sra e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2C]).unwrap()),
            "sra h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2D]).unwrap()),
            "sra l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2E]).unwrap()),
            "sra (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x2F]).unwrap()),
            "sra a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x38]).unwrap()),
            "srl b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x39]).unwrap()),
            "srl c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3A]).unwrap()),
            "srl d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3B]).unwrap()),
            "srl e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3C]).unwrap()),
            "srl h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3D]).unwrap()),
            "srl l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3E]).unwrap()),
            "srl (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x3F]).unwrap()),
            "srl a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x40]).unwrap()),
            "bit 0, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x41]).unwrap()),
            "bit 0, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x42]).unwrap()),
            "bit 0, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x43]).unwrap()),
            "bit 0, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x44]).unwrap()),
            "bit 0, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x45]).unwrap()),
            "bit 0, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x46]).unwrap()),
            "bit 0, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x47]).unwrap()),
            "bit 0, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x48]).unwrap()),
            "bit 1, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x49]).unwrap()),
            "bit 1, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4A]).unwrap()),
            "bit 1, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4B]).unwrap()),
            "bit 1, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4C]).unwrap()),
            "bit 1, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4D]).unwrap()),
            "bit 1, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4E]).unwrap()),
            "bit 1, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x4F]).unwrap()),
            "bit 1, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x50]).unwrap()),
            "bit 2, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x51]).unwrap()),
            "bit 2, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x52]).unwrap()),
            "bit 2, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x53]).unwrap()),
            "bit 2, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x54]).unwrap()),
            "bit 2, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x55]).unwrap()),
            "bit 2, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x56]).unwrap()),
            "bit 2, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x57]).unwrap()),
            "bit 2, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x58]).unwrap()),
            "bit 3, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x59]).unwrap()),
            "bit 3, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5A]).unwrap()),
            "bit 3, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5B]).unwrap()),
            "bit 3, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5C]).unwrap()),
            "bit 3, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5D]).unwrap()),
            "bit 3, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5E]).unwrap()),
            "bit 3, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x5F]).unwrap()),
            "bit 3, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x60]).unwrap()),
            "bit 4, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x61]).unwrap()),
            "bit 4, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x62]).unwrap()),
            "bit 4, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x63]).unwrap()),
            "bit 4, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x64]).unwrap()),
            "bit 4, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x65]).unwrap()),
            "bit 4, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x66]).unwrap()),
            "bit 4, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x67]).unwrap()),
            "bit 4, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x68]).unwrap()),
            "bit 5, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x69]).unwrap()),
            "bit 5, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6A]).unwrap()),
            "bit 5, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6B]).unwrap()),
            "bit 5, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6C]).unwrap()),
            "bit 5, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6D]).unwrap()),
            "bit 5, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6E]).unwrap()),
            "bit 5, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x6F]).unwrap()),
            "bit 5, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x70]).unwrap()),
            "bit 6, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x71]).unwrap()),
            "bit 6, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x72]).unwrap()),
            "bit 6, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x73]).unwrap()),
            "bit 6, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x74]).unwrap()),
            "bit 6, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x75]).unwrap()),
            "bit 6, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x76]).unwrap()),
            "bit 6, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x77]).unwrap()),
            "bit 6, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x78]).unwrap()),
            "bit 7, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x79]).unwrap()),
            "bit 7, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7A]).unwrap()),
            "bit 7, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7B]).unwrap()),
            "bit 7, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7C]).unwrap()),
            "bit 7, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7D]).unwrap()),
            "bit 7, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7E]).unwrap()),
            "bit 7, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x7F]).unwrap()),
            "bit 7, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x80]).unwrap()),
            "res 0, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x81]).unwrap()),
            "res 0, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x82]).unwrap()),
            "res 0, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x83]).unwrap()),
            "res 0, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x84]).unwrap()),
            "res 0, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x85]).unwrap()),
            "res 0, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x86]).unwrap()),
            "res 0, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x87]).unwrap()),
            "res 0, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x88]).unwrap()),
            "res 1, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x89]).unwrap()),
            "res 1, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8A]).unwrap()),
            "res 1, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8B]).unwrap()),
            "res 1, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8C]).unwrap()),
            "res 1, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8D]).unwrap()),
            "res 1, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8E]).unwrap()),
            "res 1, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x8F]).unwrap()),
            "res 1, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x90]).unwrap()),
            "res 2, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x91]).unwrap()),
            "res 2, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x92]).unwrap()),
            "res 2, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x93]).unwrap()),
            "res 2, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x94]).unwrap()),
            "res 2, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x95]).unwrap()),
            "res 2, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x96]).unwrap()),
            "res 2, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x97]).unwrap()),
            "res 2, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x98]).unwrap()),
            "res 3, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x99]).unwrap()),
            "res 3, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9A]).unwrap()),
            "res 3, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9B]).unwrap()),
            "res 3, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9C]).unwrap()),
            "res 3, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9D]).unwrap()),
            "res 3, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9E]).unwrap()),
            "res 3, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0x9F]).unwrap()),
            "res 3, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA0]).unwrap()),
            "res 4, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA1]).unwrap()),
            "res 4, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA2]).unwrap()),
            "res 4, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA3]).unwrap()),
            "res 4, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA4]).unwrap()),
            "res 4, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA5]).unwrap()),
            "res 4, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA6]).unwrap()),
            "res 4, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA7]).unwrap()),
            "res 4, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA8]).unwrap()),
            "res 5, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xA9]).unwrap()),
            "res 5, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAA]).unwrap()),
            "res 5, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAB]).unwrap()),
            "res 5, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAC]).unwrap()),
            "res 5, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAD]).unwrap()),
            "res 5, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAE]).unwrap()),
            "res 5, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xAF]).unwrap()),
            "res 5, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB0]).unwrap()),
            "res 6, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB1]).unwrap()),
            "res 6, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB2]).unwrap()),
            "res 6, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB3]).unwrap()),
            "res 6, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB4]).unwrap()),
            "res 6, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB5]).unwrap()),
            "res 6, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB6]).unwrap()),
            "res 6, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB7]).unwrap()),
            "res 6, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB8]).unwrap()),
            "res 7, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xB9]).unwrap()),
            "res 7, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBA]).unwrap()),
            "res 7, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBB]).unwrap()),
            "res 7, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBC]).unwrap()),
            "res 7, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBD]).unwrap()),
            "res 7, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBE]).unwrap()),
            "res 7, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xBF]).unwrap()),
            "res 7, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC0]).unwrap()),
            "set 0, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC1]).unwrap()),
            "set 0, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC2]).unwrap()),
            "set 0, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC3]).unwrap()),
            "set 0, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC4]).unwrap()),
            "set 0, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC5]).unwrap()),
            "set 0, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC6]).unwrap()),
            "set 0, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC7]).unwrap()),
            "set 0, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC8]).unwrap()),
            "set 1, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xC9]).unwrap()),
            "set 1, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCA]).unwrap()),
            "set 1, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCB]).unwrap()),
            "set 1, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCC]).unwrap()),
            "set 1, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCD]).unwrap()),
            "set 1, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCE]).unwrap()),
            "set 1, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xCF]).unwrap()),
            "set 1, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD0]).unwrap()),
            "set 2, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD1]).unwrap()),
            "set 2, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD2]).unwrap()),
            "set 2, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD3]).unwrap()),
            "set 2, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD4]).unwrap()),
            "set 2, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD5]).unwrap()),
            "set 2, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD6]).unwrap()),
            "set 2, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD7]).unwrap()),
            "set 2, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD8]).unwrap()),
            "set 3, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xD9]).unwrap()),
            "set 3, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDA]).unwrap()),
            "set 3, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDB]).unwrap()),
            "set 3, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDC]).unwrap()),
            "set 3, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDD]).unwrap()),
            "set 3, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDE]).unwrap()),
            "set 3, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xDF]).unwrap()),
            "set 3, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE0]).unwrap()),
            "set 4, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE1]).unwrap()),
            "set 4, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE2]).unwrap()),
            "set 4, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE3]).unwrap()),
            "set 4, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE4]).unwrap()),
            "set 4, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE5]).unwrap()),
            "set 4, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE6]).unwrap()),
            "set 4, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE7]).unwrap()),
            "set 4, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE8]).unwrap()),
            "set 5, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xE9]).unwrap()),
            "set 5, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xEA]).unwrap()),
            "set 5, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xEB]).unwrap()),
            "set 5, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xEC]).unwrap()),
            "set 5, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xED]).unwrap()),
            "set 5, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xEE]).unwrap()),
            "set 5, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xEF]).unwrap()),
            "set 5, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF0]).unwrap()),
            "set 6, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF1]).unwrap()),
            "set 6, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF2]).unwrap()),
            "set 6, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF3]).unwrap()),
            "set 6, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF4]).unwrap()),
            "set 6, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF5]).unwrap()),
            "set 6, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF6]).unwrap()),
            "set 6, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF7]).unwrap()),
            "set 6, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF8]).unwrap()),
            "set 7, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xF9]).unwrap()),
            "set 7, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFA]).unwrap()),
            "set 7, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFB]).unwrap()),
            "set 7, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFC]).unwrap()),
            "set 7, h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFD]).unwrap()),
            "set 7, l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFE]).unwrap()),
            "set 7, (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xcb, 0xFF]).unwrap()),
            "set 7, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xCC, 0x34, 0x12]).unwrap()),
            "call z, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xCD, 0x34, 0x12]).unwrap()),
            "call 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xCE, 0x45]).unwrap()),
            "adc a, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xCF]).unwrap()),
            "rst $08"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD0]).unwrap()),
            "ret nc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD1]).unwrap()),
            "pop de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD2, 0x34, 0x12]).unwrap()),
            "jp nc, 1234h"
        );
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xD3 ,0x45]).unwrap()), "out (45h), a");
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD4, 0x34, 0x12]).unwrap()),
            "call nc, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD5]).unwrap()),
            "push de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD6, 0x45]).unwrap()),
            "sub a, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD7]).unwrap()),
            "rst $10"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xD8]).unwrap()),
            "ret c"
        );
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xD9]).unwrap()), "exx");
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDA, 0x34, 0x12]).unwrap()),
            "jp c, 1234h"
        );
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xDB ,0x45])), "in a, (45h)");
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDC, 0x34, 0x12]).unwrap()),
            "call c, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x09]).unwrap()),
            "add ix, bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x19]).unwrap()),
            "add ix, de"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xdd, 0x21, 0x34, 0x12]).unwrap()
            ),
            "ld ix, 1234h",
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xdd, 0x22, 0x34, 0x12]).unwrap()
            ),
            "ld (1234h), ix"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x23]).unwrap()),
            "inc ix"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x24]).unwrap()),
            "inc ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x25]).unwrap()),
            "dec ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x26, 0x45]).unwrap()),
            "ld ixh, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x29]).unwrap()),
            "add ix, ix"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xdd, 0x2a, 0x34, 0x12]).unwrap()
            ),
            "ld ix, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x2b]).unwrap()),
            "dec ix"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x2c]).unwrap()),
            "inc ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x2d]).unwrap()),
            "dec ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x2e, 0x45]).unwrap()),
            "ld ixl, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x34, 0x45]).unwrap()),
            "inc (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x35, 0x45]).unwrap()),
            "dec (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x39]).unwrap()),
            "add ix, sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x44]).unwrap()),
            "ld b, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x45]).unwrap()),
            "ld b, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x46, 0x45]).unwrap()),
            "ld b, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x4c]).unwrap()),
            "ld c, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x4d]).unwrap()),
            "ld c, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x4e, 0x45]).unwrap()),
            "ld c, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x54]).unwrap()),
            "ld d, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x55]).unwrap()),
            "ld d, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x5e, 0x45]).unwrap()),
            "ld e, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x60]).unwrap()),
            "ld ixh, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x61]).unwrap()),
            "ld ixh, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x62]).unwrap()),
            "ld ixh, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x63]).unwrap()),
            "ld ixh, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x65]).unwrap()),
            "ld ixh, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x66, 0x45]).unwrap()),
            "ld h, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x67]).unwrap()),
            "ld ixh, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x68]).unwrap()),
            "ld ixl, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x69]).unwrap()),
            "ld ixl, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x6a]).unwrap()),
            "ld ixl, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x6b]).unwrap()),
            "ld ixl, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x6c]).unwrap()),
            "ld ixl, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x6f]).unwrap()),
            "ld ixl, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x70, 0x45]).unwrap()),
            "ld (ix+45h), b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x71, 0x45]).unwrap()),
            "ld (ix+45h), c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xdd, 0x72, 0x45]).unwrap()),
            "ld (ix+45h), d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x73, 0x45]).unwrap()),
            "ld (ix+45h), e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x74, 0x45]).unwrap()),
            "ld (ix+45h), ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x75, 0x45]).unwrap()),
            "ld (ix+45h), ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x77, 0x45]).unwrap()),
            "ld (ix+45h), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x7C]).unwrap()),
            "ld a, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x7D]).unwrap()),
            "ld a, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x7E, 0x45]).unwrap()),
            "ld a, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x84]).unwrap()),
            "add a, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x85]).unwrap()),
            "add a, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x86, 0x45]).unwrap()),
            "add a, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x8C]).unwrap()),
            "adc a, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x8D]).unwrap()),
            "adc a, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x8E, 0x45]).unwrap()),
            "adc a, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x94]).unwrap()),
            "sub a, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x95]).unwrap()),
            "sub a, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x96, 0x45]).unwrap()),
            "sub a, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x9C]).unwrap()),
            "sbc a, ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x9D]).unwrap()),
            "sbc a, ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0x9E, 0x45]).unwrap()),
            "sbc a, (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xA4]).unwrap()),
            "and ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xA5]).unwrap()),
            "and ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xA6, 0x45]).unwrap()),
            "and (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xAC]).unwrap()),
            "xor ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xAD]).unwrap()),
            "xor ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xAE, 0x45]).unwrap()),
            "xor (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xB4]).unwrap()),
            "or ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xB5]).unwrap()),
            "or ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xB6, 0x45]).unwrap()),
            "or (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xBC]).unwrap()),
            "cp ixh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xBD]).unwrap()),
            "cp ixl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xBE, 0x45]).unwrap()),
            "cp (ix+45h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xE1]).unwrap()),
            "pop ix"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xE5]).unwrap()),
            "push ix"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDD, 0xE9]).unwrap()),
            "jp (ix)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDE, 0x45]).unwrap()),
            "sbc a, 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xDF]).unwrap()),
            "rst $18"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE0]).unwrap()),
            "ret po"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE1]).unwrap()),
            "pop hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE2, 0x34, 0x12]).unwrap()),
            "jp po, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE3]).unwrap()),
            "ex (sp), hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE4, 0x34, 0x12]).unwrap()),
            "call po, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE5]).unwrap()),
            "push hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE6, 0x45]).unwrap()),
            "and 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE7]).unwrap()),
            "rst $20"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE8]).unwrap()),
            "ret pe"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xE9]).unwrap()),
            "jp (hl)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xEA, 0x34, 0x12]).unwrap()),
            "jp pe, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xEB]).unwrap()),
            "ex de, hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xEC, 0x34, 0x12]).unwrap()),
            "call pe, 1234h"
        );

        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x40]).unwrap()),
            "in b, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x41]).unwrap()),
            "out (c), b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x42]).unwrap()),
            "sbc hl, bc"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x43, 0x34, 0x12]).unwrap()
            ),
            "ld (1234h), bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x44]).unwrap()),
            "neg"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x45]).unwrap()),
            "retn"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x46]).unwrap()),
            "im 0"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x47]).unwrap()),
            "ld i, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x48]).unwrap()),
            "in c, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x49]).unwrap()),
            "out (c), c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x4A]).unwrap()),
            "adc hl, bc"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x4B, 0x34, 0x12]).unwrap()
            ),
            "ld bc, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x4D]).unwrap()),
            "reti"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x4F]).unwrap()),
            "ld r, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x50]).unwrap()),
            "in d, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x51]).unwrap()),
            "out (c), d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x52]).unwrap()),
            "sbc hl, de"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x53, 0x34, 0x12]).unwrap()
            ),
            "ld (1234h), de"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x56]).unwrap()),
            "im 1"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x57]).unwrap()),
            "ld a, i"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x58]).unwrap()),
            "in e, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x59]).unwrap()),
            "out (c), e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x5A]).unwrap()),
            "adc hl, de"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x5B, 0x34, 0x12]).unwrap()
            ),
            "ld de, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x5E]).unwrap()),
            "im 2"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x5F]).unwrap()),
            "ld a, r"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x60]).unwrap()),
            "in h, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x61]).unwrap()),
            "out (c), h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x62]).unwrap()),
            "sbc hl, hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x67]).unwrap()),
            "rrd"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x68]).unwrap()),
            "in l, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x69]).unwrap()),
            "out (c), l"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x6A]).unwrap()),
            "adc hl, hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x6F]).unwrap()),
            "rld"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x70]).unwrap()),
            "in f, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x71]).unwrap()),
            "out (c), f"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x72]).unwrap()),
            "sbc hl, sp"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x73, 0x34, 0x12]).unwrap()
            ),
            "ld (1234h), sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x78]).unwrap()),
            "in a, (c)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x79]).unwrap()),
            "out (c), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0x7A]).unwrap()),
            "adc hl, sp"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xED, 0x7B, 0x34, 0x12]).unwrap()
            ),
            "ld sp, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA0]).unwrap()),
            "ldi"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA1]).unwrap()),
            "cpi"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA2]).unwrap()),
            "ini"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA3]).unwrap()),
            "oti"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA8]).unwrap()),
            "ldd"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xA9]).unwrap()),
            "cpd"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xAA]).unwrap()),
            "ind"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xAB]).unwrap()),
            "otd"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB0]).unwrap()),
            "ldir"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB1]).unwrap()),
            "cpir"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB2]).unwrap()),
            "inir"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB3]).unwrap()),
            "otir"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB8]).unwrap()),
            "lddr"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xB9]).unwrap()),
            "cpdr"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xBA]).unwrap()),
            "indr"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xED, 0xBB]).unwrap()),
            "otdr"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xEE, 0x45]).unwrap()),
            "xor 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xEF]).unwrap()),
            "rst $28"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF0]).unwrap()),
            "ret p"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF1]).unwrap()),
            "pop af"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF2, 0x34, 0x12]).unwrap()),
            "jp p, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF3]).unwrap()),
            "di"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF4, 0x34, 0x12]).unwrap()),
            "call p, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF5]).unwrap()),
            "push af"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF6, 0x45]).unwrap()),
            "or 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF7]).unwrap()),
            "rst $30"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF8]).unwrap()),
            "ret m"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xF9]).unwrap()),
            "ld sp, hl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFA, 0x34, 0x12]).unwrap()),
            "jp m, 1234h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFB]).unwrap()),
            "ei"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFC, 0x34, 0x12]).unwrap()),
            "call m, 1234h"
        );

        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x09]).unwrap()),
            "add iy, bc"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x19]).unwrap()),
            "add iy, de"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xFD, 0x21, 0x34, 0x12]).unwrap()
            ),
            "ld iy, 1234h"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xFD, 0x22, 0x34, 0x12]).unwrap()
            ),
            "ld (1234h), iy"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x23]).unwrap()),
            "inc iy"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x24]).unwrap()),
            "inc iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x25]).unwrap()),
            "dec iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x26, 0x69]).unwrap()),
            "ld iyh, 69h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x29]).unwrap()),
            "add iy, iy"
        );
        assert_eq!(
            format!(
                "{}",
                Instruction::from_bytes(&[0xFD, 0x2A, 0x34, 0x12]).unwrap()
            ),
            "ld iy, (1234h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x2B]).unwrap()),
            "dec iy"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x2C]).unwrap()),
            "inc iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x2D]).unwrap()),
            "dec iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x2E, 0x69]).unwrap()),
            "ld iyl, 69h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x34, 0x69]).unwrap()),
            "inc (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x35, 0x69]).unwrap()),
            "dec (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x39]).unwrap()),
            "add iy, sp"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x44]).unwrap()),
            "ld b, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x45]).unwrap()),
            "ld b, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x46, 0x69]).unwrap()),
            "ld b, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x4C]).unwrap()),
            "ld c, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x4D]).unwrap()),
            "ld c, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x4E, 0x69]).unwrap()),
            "ld c, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x54]).unwrap()),
            "ld d, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x55]).unwrap()),
            "ld d, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x5E, 0x69]).unwrap()),
            "ld e, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x60]).unwrap()),
            "ld iyh, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x61]).unwrap()),
            "ld iyh, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x62]).unwrap()),
            "ld iyh, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x63]).unwrap()),
            "ld iyh, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x65]).unwrap()),
            "ld iyh, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x66, 0x69]).unwrap()),
            "ld h, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x67]).unwrap()),
            "ld iyh, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x68]).unwrap()),
            "ld iyl, b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x69]).unwrap()),
            "ld iyl, c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x6A]).unwrap()),
            "ld iyl, d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x6B]).unwrap()),
            "ld iyl, e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x6C]).unwrap()),
            "ld iyl, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x6F]).unwrap()),
            "ld iyl, a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x70, 0x69]).unwrap()),
            "ld (iy+69h), b"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x71, 0x69]).unwrap()),
            "ld (iy+69h), c"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x72, 0x69]).unwrap()),
            "ld (iy+69h), d"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x73, 0x69]).unwrap()),
            "ld (iy+69h), e"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x77, 0x69]).unwrap()),
            "ld (iy+69h), a"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x7C]).unwrap()),
            "ld a, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x7D]).unwrap()),
            "ld a, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x7E, 0x69]).unwrap()),
            "ld a, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x84]).unwrap()),
            "add a, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x85]).unwrap()),
            "add a, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x86, 0x69]).unwrap()),
            "add a, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x8C]).unwrap()),
            "adc a, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x8D]).unwrap()),
            "adc a, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x8E, 0x69]).unwrap()),
            "adc a, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x94]).unwrap()),
            "sub a, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x95]).unwrap()),
            "sub a, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x96, 0x69]).unwrap()),
            "sub a, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x9C]).unwrap()),
            "sbc a, iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x9D]).unwrap()),
            "sbc a, iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0x9E, 0x69]).unwrap()),
            "sbc a, (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xA4]).unwrap()),
            "and iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xA5]).unwrap()),
            "and iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xA6, 0x69]).unwrap()),
            "and (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xAC]).unwrap()),
            "xor iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xAD]).unwrap()),
            "xor iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xAE, 0x69]).unwrap()),
            "xor (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xB4]).unwrap()),
            "or iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xB5]).unwrap()),
            "or iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xB6, 0x69]).unwrap()),
            "or (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xBC]).unwrap()),
            "cp iyh"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xBD]).unwrap()),
            "cp iyl"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xBE, 0x69]).unwrap()),
            "cp (iy+69h)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xE1]).unwrap()),
            "pop iy"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xE5]).unwrap()),
            "push iy"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFD, 0xE9]).unwrap()),
            "jp (iy)"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFE, 0x45]).unwrap()),
            "cp 45h"
        );
        assert_eq!(
            format!("{}", Instruction::from_bytes(&[0xFF]).unwrap()),
            "rst $38"
        );
    }
}
