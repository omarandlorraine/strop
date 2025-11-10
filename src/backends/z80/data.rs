use crate::backends::x80::data::InstructionData;
use crate::backends::x80::data::ReadWrite;

// A table containing only what's different between unprefixed Z80 instruction set and the 8080
// instruction set.
pub static UNPREFIXED: [Option<InstructionData>; 256] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // EX AF, AF'
        mnemonic: "ex",
        flow_control: false,
        opcode: 0x08,
        bytes: 1,
        cycles: 20,
        zero: ReadWrite::N,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::N,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::R,
        i: ReadWrite::N,
        operands: ["af", "af'", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // djnz e8
        mnemonic: "djnz",
        flow_control: true,
        opcode: 0x10,
        bytes: 2,
        cycles: 4,
        zero: ReadWrite::N,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::N,
        a: ReadWrite::N,
        b: ReadWrite::Rmw,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["e8", "", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // JR e8
        mnemonic: "jr",
        flow_control: true,
        opcode: 0x18,
        bytes: 2,
        cycles: 12,
        zero: ReadWrite::N,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::N,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["e8", "", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // JR NZ e8
        mnemonic: "jr",
        flow_control: true,
        opcode: 0x20,
        bytes: 2,
        cycles: 12,
        zero: ReadWrite::R,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::N,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["nz", "e8", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // JR Z e8
        mnemonic: "jr",
        flow_control: true,
        opcode: 0x28,
        bytes: 2,
        cycles: 12,
        zero: ReadWrite::R,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::N,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["z", "e8", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // JR NC e8
        mnemonic: "jr",
        flow_control: true,
        opcode: 0x30,
        bytes: 2,
        cycles: 12,
        zero: ReadWrite::N,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::R,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["nc", "e8", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(InstructionData {
        // JR C e8
        mnemonic: "jr",
        flow_control: true,
        opcode: 0x38,
        bytes: 2,
        cycles: 12,
        zero: ReadWrite::N,
        negative: ReadWrite::N,
        half_carry: ReadWrite::N,
        carry: ReadWrite::R,
        a: ReadWrite::N,
        b: ReadWrite::N,
        c: ReadWrite::N,
        d: ReadWrite::N,
        e: ReadWrite::N,
        h: ReadWrite::N,
        l: ReadWrite::N,
        r: ReadWrite::N,
        ixh: ReadWrite::N,
        ixl: ReadWrite::N,
        iyh: ReadWrite::N,
        iyl: ReadWrite::N,
        sp: ReadWrite::N,
        i: ReadWrite::N,
        operands: ["c", "e8", ""],
    }),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

pub static EDPREFIXED: [Option<InstructionData>; 256] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];
pub static DDCBPREFIXED: [Option<InstructionData>; 256] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];
pub static FDCBPREFIXED: [Option<InstructionData>; 256] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];
pub static DDPREFIXED: [Option<InstructionData>; 256] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];
pub static FDPREFIXED: [Option<InstructionData>; 256] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
];

#[cfg(test)]
mod test {
    #[test]
    fn dedup8080() {
        for i in 0..=255 {
            if let Some(ref i8080) = crate::backends::i8080::data::UNPREFIXED[i] {
                let Some(ref z80) = crate::backends::z80::data::UNPREFIXED[i] else {
                    continue;
                };
                assert_ne!(i8080, z80, "{z80:?}");
            }
        }
    }

    #[test]
    fn dasm() {
        use crate::Instruction as _;
        use crate::backends::z80::Instruction;
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x00]).unwrap()), "nop");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x01 ,0x34 ,0x12]).unwrap()), "ld bc, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x02]).unwrap()), "ld (bc), a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x03]).unwrap()), "inc bc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x04]).unwrap()), "inc b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x05]).unwrap()), "dec b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x06 ,0x45]).unwrap()), "ld b, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x07]).unwrap()), "rlca");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x08]).unwrap()), "ex af, af'");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x09]).unwrap()), "add hl, bc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0A]).unwrap()), "ld a, (bc)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0B]).unwrap()), "dec bc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0C]).unwrap()), "inc c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0D]).unwrap()), "dec c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0E ,0x45]).unwrap()), "ld c, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x0F]).unwrap()), "rrca");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x10 ,0x45]).unwrap()), "djnz 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x11 ,0x34 ,0x12]).unwrap()), "ld de, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x12]).unwrap()), "ld (de), a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x13]).unwrap()), "inc de");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x14]).unwrap()), "inc d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x15]).unwrap()), "dec d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x16 ,0x45]).unwrap()), "ld d, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x17]).unwrap()), "rla");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x18 ,0x45]).unwrap()), "jr 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x19]).unwrap()), "add hl, de");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1A]).unwrap()), "ld a, (de)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1B]).unwrap()), "dec de");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1C]).unwrap()), "inc e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1D]).unwrap()), "dec e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1E ,0x45]).unwrap()), "ld e, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x1F]).unwrap()), "rra");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x20 ,0x45]).unwrap()), "jr nz, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x21 ,0x34 ,0x12]).unwrap()), "ld hl, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x22 ,0x34 ,0x12]).unwrap()), "ld (1234h), hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x23]).unwrap()), "inc hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x24]).unwrap()), "inc h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x25]).unwrap()), "dec h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x26 ,0x45]).unwrap()), "ld h, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x27]).unwrap()), "daa");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x28 ,0x45]).unwrap()), "jr z, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x29]).unwrap()), "add hl, hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2A ,0x34 ,0x12]).unwrap()), "ld hl, (1234h)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2B]).unwrap()), "dec hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2C]).unwrap()), "inc l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2D]).unwrap()), "dec l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2E ,0x45]).unwrap()), "ld l, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x2F]).unwrap()), "cpl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x30 ,0x45]).unwrap()), "jr nc, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x31 ,0x34 ,0x12]).unwrap()), "ld sp, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x32 ,0x34 ,0x12]).unwrap()), "ld (1234h), a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x33]).unwrap()), "inc sp");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x34]).unwrap()), "inc (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x35]).unwrap()), "dec (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x36 ,0x45]).unwrap()), "ld (hl), 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x37]).unwrap()), "scf");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x38 ,0x45]).unwrap()), "jr c, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x39]).unwrap()), "add hl, sp");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3A ,0x34 ,0x12]).unwrap()), "ld a, (1234h)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3B]).unwrap()), "dec sp");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3C]).unwrap()), "inc a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3D]).unwrap()), "dec a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3E ,0x45]).unwrap()), "ld a, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x3F]).unwrap()), "ccf");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x41]).unwrap()), "ld b, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x42]).unwrap()), "ld b, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x43]).unwrap()), "ld b, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x44]).unwrap()), "ld b, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x45]).unwrap()), "ld b, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x46]).unwrap()), "ld b, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x47]).unwrap()), "ld b, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x48]).unwrap()), "ld c, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4A]).unwrap()), "ld c, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4B]).unwrap()), "ld c, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4C]).unwrap()), "ld c, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4D]).unwrap()), "ld c, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4E]).unwrap()), "ld c, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x4F]).unwrap()), "ld c, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x50]).unwrap()), "ld d, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x51]).unwrap()), "ld d, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x53]).unwrap()), "ld d, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x54]).unwrap()), "ld d, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x55]).unwrap()), "ld d, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x56]).unwrap()), "ld d, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x57]).unwrap()), "ld d, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x58]).unwrap()), "ld e, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x59]).unwrap()), "ld e, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x5A]).unwrap()), "ld e, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x5C]).unwrap()), "ld e, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x5D]).unwrap()), "ld e, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x5E]).unwrap()), "ld e, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x5F]).unwrap()), "ld e, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x60]).unwrap()), "ld h, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x61]).unwrap()), "ld h, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x62]).unwrap()), "ld h, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x63]).unwrap()), "ld h, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x65]).unwrap()), "ld h, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x66]).unwrap()), "ld h, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x67]).unwrap()), "ld h, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x68]).unwrap()), "ld l, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x69]).unwrap()), "ld l, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x6A]).unwrap()), "ld l, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x6B]).unwrap()), "ld l, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x6C]).unwrap()), "ld l, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x6E]).unwrap()), "ld l, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x6F]).unwrap()), "ld l, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x70]).unwrap()), "ld (hl), b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x71]).unwrap()), "ld (hl), c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x72]).unwrap()), "ld (hl), d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x73]).unwrap()), "ld (hl), e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x74]).unwrap()), "ld (hl), h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x75]).unwrap()), "ld (hl), l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x76]).unwrap()), "halt");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x77]).unwrap()), "ld (hl), a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x78]).unwrap()), "ld a, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x79]).unwrap()), "ld a, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x7A]).unwrap()), "ld a, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x7B]).unwrap()), "ld a, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x7C]).unwrap()), "ld a, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x7D]).unwrap()), "ld a, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x7E]).unwrap()), "ld a, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x80]).unwrap()), "add a, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x81]).unwrap()), "add a, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x82]).unwrap()), "add a, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x83]).unwrap()), "add a, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x84]).unwrap()), "add a, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x85]).unwrap()), "add a, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x86]).unwrap()), "add a, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x87]).unwrap()), "add a, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x88]).unwrap()), "adc a, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x89]).unwrap()), "adc a, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8A]).unwrap()), "adc a, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8B]).unwrap()), "adc a, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8C]).unwrap()), "adc a, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8D]).unwrap()), "adc a, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8E]).unwrap()), "adc a, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x8F]).unwrap()), "adc a, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x90]).unwrap()), "sub a, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x91]).unwrap()), "sub a, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x92]).unwrap()), "sub a, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x93]).unwrap()), "sub a, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x94]).unwrap()), "sub a, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x95]).unwrap()), "sub a, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x96]).unwrap()), "sub a, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x97]).unwrap()), "sub a, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x98]).unwrap()), "sbc a, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x99]).unwrap()), "sbc a, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9A]).unwrap()), "sbc a, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9B]).unwrap()), "sbc a, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9C]).unwrap()), "sbc a, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9D]).unwrap()), "sbc a, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9E]).unwrap()), "sbc a, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0x9F]).unwrap()), "sbc a, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA0]).unwrap()), "and b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA1]).unwrap()), "and c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA2]).unwrap()), "and d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA3]).unwrap()), "and e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA4]).unwrap()), "and h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA5]).unwrap()), "and l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA6]).unwrap()), "and (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA7]).unwrap()), "and a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA8]).unwrap()), "xor b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xA9]).unwrap()), "xor c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAA]).unwrap()), "xor d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAB]).unwrap()), "xor e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAC]).unwrap()), "xor h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAD]).unwrap()), "xor l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAE]).unwrap()), "xor (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xAF]).unwrap()), "xor a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB0]).unwrap()), "or b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB1]).unwrap()), "or c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB2]).unwrap()), "or d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB3]).unwrap()), "or e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB4]).unwrap()), "or h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB5]).unwrap()), "or l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB6]).unwrap()), "or (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB7]).unwrap()), "or a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB8]).unwrap()), "cp b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xB9]).unwrap()), "cp c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBA]).unwrap()), "cp d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBB]).unwrap()), "cp e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBC]).unwrap()), "cp h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBD]).unwrap()), "cp l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBE]).unwrap()), "cp (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xBF]).unwrap()), "cp a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC0]).unwrap()), "ret nz");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC1]).unwrap()), "pop bc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC2 ,0x34 ,0x12]).unwrap()), "jp nz, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC3 ,0x34 ,0x12]).unwrap()), "jp 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC4 ,0x34 ,0x12]).unwrap()), "call nz, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC5]).unwrap()), "push bc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC6 ,0x45]).unwrap()), "add a, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC7]).unwrap()), "rst $00");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC8]).unwrap()), "ret z");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xC9]).unwrap()), "ret");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xCA ,0x34 ,0x12]).unwrap()), "jp z, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x00]).unwrap()),"rlc b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x01]).unwrap()),"rlc c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x02]).unwrap()),"rlc d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x03]).unwrap()),"rlc e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x04]).unwrap()),"rlc h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x05]).unwrap()),"rlc l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x06]).unwrap()),"rlc (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x07]).unwrap()),"rlc a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x08]).unwrap()),"rrc b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x09]).unwrap()),"rrc c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x0A]).unwrap()),"rrc d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x0B]).unwrap()),"rrc e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x0E]).unwrap()),"rrc (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x0F]).unwrap()),"rrc a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x10]).unwrap()),"rl b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x11]).unwrap()),"rl c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x12]).unwrap()),"rl d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x13]).unwrap()),"rl e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x14]).unwrap()),"rl h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x15]).unwrap()),"rl l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x16]).unwrap()),"rl (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x17]).unwrap()),"rl a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x18]).unwrap()),"rr b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x19]).unwrap()),"rr c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1A]).unwrap()),"rr d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1B]).unwrap()),"rr e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1C]).unwrap()),"rr h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1D]).unwrap()),"rr l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1E]).unwrap()),"rr (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x1F]).unwrap()),"rr a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x20]).unwrap()),"sla b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x21]).unwrap()),"sla c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x22]).unwrap()),"sla d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x23]).unwrap()),"sla e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x24]).unwrap()),"sla h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x25]).unwrap()),"sla l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x26]).unwrap()),"sla (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x27]).unwrap()),"sla a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x28]).unwrap()),"sra b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x29]).unwrap()),"sra c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2A]).unwrap()),"sra d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2B]).unwrap()),"sra e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2C]).unwrap()),"sra h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2D]).unwrap()),"sra l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2E]).unwrap()),"sra (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x2F]).unwrap()),"sra a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x38]).unwrap()),"srl b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x39]).unwrap()),"srl c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3A]).unwrap()),"srl d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3B]).unwrap()),"srl e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3C]).unwrap()),"srl h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3D]).unwrap()),"srl l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3E]).unwrap()),"srl (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x3F]).unwrap()),"srl a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x40]).unwrap()),"bit 0, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x41]).unwrap()),"bit 0, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x42]).unwrap()),"bit 0, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x43]).unwrap()),"bit 0, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x44]).unwrap()),"bit 0, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x45]).unwrap()),"bit 0, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x46]).unwrap()),"bit 0, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x47]).unwrap()),"bit 0, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x48]).unwrap()),"bit 1, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x49]).unwrap()),"bit 1, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4A]).unwrap()),"bit 1, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4B]).unwrap()),"bit 1, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4C]).unwrap()),"bit 1, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4D]).unwrap()),"bit 1, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4E]).unwrap()),"bit 1, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x4F]).unwrap()),"bit 1, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x50]).unwrap()),"bit 2, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x51]).unwrap()),"bit 2, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x52]).unwrap()),"bit 2, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x53]).unwrap()),"bit 2, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x54]).unwrap()),"bit 2, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x55]).unwrap()),"bit 2, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x56]).unwrap()),"bit 2, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x57]).unwrap()),"bit 2, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x58]).unwrap()),"bit 3, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x59]).unwrap()),"bit 3, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5A]).unwrap()),"bit 3, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5B]).unwrap()),"bit 3, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5C]).unwrap()),"bit 3, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5D]).unwrap()),"bit 3, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5E]).unwrap()),"bit 3, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x5F]).unwrap()),"bit 3, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x60]).unwrap()),"bit 4, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x61]).unwrap()),"bit 4, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x62]).unwrap()),"bit 4, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x63]).unwrap()),"bit 4, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x64]).unwrap()),"bit 4, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x65]).unwrap()),"bit 4, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x66]).unwrap()),"bit 4, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x67]).unwrap()),"bit 4, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x68]).unwrap()),"bit 5, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x69]).unwrap()),"bit 5, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6A]).unwrap()),"bit 5, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6B]).unwrap()),"bit 5, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6C]).unwrap()),"bit 5, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6D]).unwrap()),"bit 5, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6E]).unwrap()),"bit 5, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x6F]).unwrap()),"bit 5, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x70]).unwrap()),"bit 6, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x71]).unwrap()),"bit 6, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x72]).unwrap()),"bit 6, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x73]).unwrap()),"bit 6, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x74]).unwrap()),"bit 6, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x75]).unwrap()),"bit 6, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x76]).unwrap()),"bit 6, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x77]).unwrap()),"bit 6, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x78]).unwrap()),"bit 7, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x79]).unwrap()),"bit 7, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7A]).unwrap()),"bit 7, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7B]).unwrap()),"bit 7, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7C]).unwrap()),"bit 7, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7D]).unwrap()),"bit 7, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7E]).unwrap()),"bit 7, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x7F]).unwrap()),"bit 7, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x80]).unwrap()),"res 0, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x81]).unwrap()),"res 0, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x82]).unwrap()),"res 0, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x83]).unwrap()),"res 0, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x84]).unwrap()),"res 0, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x85]).unwrap()),"res 0, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x86]).unwrap()),"res 0, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x87]).unwrap()),"res 0, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x88]).unwrap()),"res 1, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x89]).unwrap()),"res 1, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8A]).unwrap()),"res 1, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8B]).unwrap()),"res 1, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8C]).unwrap()),"res 1, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8D]).unwrap()),"res 1, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8E]).unwrap()),"res 1, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x8F]).unwrap()),"res 1, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x90]).unwrap()),"res 2, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x91]).unwrap()),"res 2, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x92]).unwrap()),"res 2, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x93]).unwrap()),"res 2, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x94]).unwrap()),"res 2, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x95]).unwrap()),"res 2, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x96]).unwrap()),"res 2, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x97]).unwrap()),"res 2, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x98]).unwrap()),"res 3, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x99]).unwrap()),"res 3, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9A]).unwrap()),"res 3, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9B]).unwrap()),"res 3, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9C]).unwrap()),"res 3, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9D]).unwrap()),"res 3, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9E]).unwrap()),"res 3, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0x9F]).unwrap()),"res 3, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA0]).unwrap()),"res 4, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA1]).unwrap()),"res 4, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA2]).unwrap()),"res 4, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA3]).unwrap()),"res 4, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA4]).unwrap()),"res 4, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA5]).unwrap()),"res 4, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA6]).unwrap()),"res 4, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA7]).unwrap()),"res 4, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA8]).unwrap()),"res 5, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xA9]).unwrap()),"res 5, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAA]).unwrap()),"res 5, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAB]).unwrap()),"res 5, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAC]).unwrap()),"res 5, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAD]).unwrap()),"res 5, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAE]).unwrap()),"res 5, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xAF]).unwrap()),"res 5, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB0]).unwrap()),"res 6, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB1]).unwrap()),"res 6, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB2]).unwrap()),"res 6, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB3]).unwrap()),"res 6, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB4]).unwrap()),"res 6, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB5]).unwrap()),"res 6, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB6]).unwrap()),"res 6, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB7]).unwrap()),"res 6, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB8]).unwrap()),"res 7, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xB9]).unwrap()),"res 7, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBA]).unwrap()),"res 7, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBB]).unwrap()),"res 7, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBC]).unwrap()),"res 7, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBD]).unwrap()),"res 7, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBE]).unwrap()),"res 7, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xBF]).unwrap()),"res 7, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC0]).unwrap()),"set 0, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC1]).unwrap()),"set 0, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC2]).unwrap()),"set 0, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC3]).unwrap()),"set 0, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC4]).unwrap()),"set 0, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC5]).unwrap()),"set 0, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC6]).unwrap()),"set 0, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC7]).unwrap()),"set 0, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC8]).unwrap()),"set 1, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xC9]).unwrap()),"set 1, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCA]).unwrap()),"set 1, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCB]).unwrap()),"set 1, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCC]).unwrap()),"set 1, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCD]).unwrap()),"set 1, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCE]).unwrap()),"set 1, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xCF]).unwrap()),"set 1, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD0]).unwrap()),"set 2, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD1]).unwrap()),"set 2, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD2]).unwrap()),"set 2, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD3]).unwrap()),"set 2, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD4]).unwrap()),"set 2, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD5]).unwrap()),"set 2, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD6]).unwrap()),"set 2, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD7]).unwrap()),"set 2, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD8]).unwrap()),"set 3, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xD9]).unwrap()),"set 3, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDA]).unwrap()),"set 3, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDB]).unwrap()),"set 3, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDC]).unwrap()),"set 3, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDD]).unwrap()),"set 3, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDE]).unwrap()),"set 3, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xDF]).unwrap()),"set 3, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE0]).unwrap()),"set 4, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE1]).unwrap()),"set 4, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE2]).unwrap()),"set 4, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE3]).unwrap()),"set 4, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE4]).unwrap()),"set 4, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE5]).unwrap()),"set 4, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE6]).unwrap()),"set 4, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE7]).unwrap()),"set 4, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE8]).unwrap()),"set 5, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xE9]).unwrap()),"set 5, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xEA]).unwrap()),"set 5, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xEB]).unwrap()),"set 5, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xEC]).unwrap()),"set 5, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xED]).unwrap()),"set 5, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xEE]).unwrap()),"set 5, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xEF]).unwrap()),"set 5, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF0]).unwrap()),"set 6, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF1]).unwrap()),"set 6, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF2]).unwrap()),"set 6, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF3]).unwrap()),"set 6, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF4]).unwrap()),"set 6, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF5]).unwrap()),"set 6, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF6]).unwrap()),"set 6, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF7]).unwrap()),"set 6, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF8]).unwrap()),"set 7, b");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xF9]).unwrap()),"set 7, c");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFA]).unwrap()),"set 7, d");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFB]).unwrap()),"set 7, e");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFC]).unwrap()),"set 7, h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFD]).unwrap()),"set 7, l");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFE]).unwrap()),"set 7, (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xcb ,0xFF]).unwrap()),"set 7, a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xCC ,0x34 ,0x12]).unwrap()), "call z, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xCD ,0x34 ,0x12]).unwrap()), "call 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xCE ,0x45]).unwrap()), "adc a, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xCF]).unwrap()), "rst $08");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD0]).unwrap()), "ret nc");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD1]).unwrap()), "pop de");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD2 ,0x34 ,0x12]).unwrap()), "jp nc, 1234h");
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xD3 ,0x45]).unwrap()), "out (45h), a");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD4 ,0x34 ,0x12]).unwrap()), "call nc, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD5]).unwrap()), "push de");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD6 ,0x45]).unwrap()), "sub a, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD7]).unwrap()), "rst $10");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xD8]).unwrap()), "ret c");
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xD9]).unwrap()), "exx");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDA ,0x34 ,0x12]).unwrap()), "jp c, 1234h");
        //assert_eq!(format!("{}", Instruction::from_bytes(&[0xDB ,0x45])), "in a, (45h)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDC ,0x34 ,0x12]).unwrap()), "call c, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x09]).unwrap()), "ADD IX,BC");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x19]).unwrap()), "ADD IX,DE");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x21 ,0x34,0x12]).unwrap()), "LD IX,HHLL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x22 ,0x34,0x12]).unwrap()), "LD (HHLL),IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x23]).unwrap()), "INC IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x24]).unwrap()), "INC IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x25]).unwrap()), "DEC IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x26 ,0x45]).unwrap()), "LD IXH,NN");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x29]).unwrap()), "ADD IX,IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x2a ,0x34,0x12]).unwrap()), "LD IX,(HHLL)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x2b]).unwrap()), "DEC IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x2c]).unwrap()), "INC IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x2d]).unwrap()), "DEC IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x2e ,0x45]).unwrap()), "LD IXL,NN");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x34 ,0x45]).unwrap()), "INC (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x35 ,0x45]).unwrap()), "DEC (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x39]).unwrap()), "ADD IX,SP");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x44]).unwrap()), "LD B,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x45]).unwrap()), "LD B,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x46 ,0x45]).unwrap()), "LD B,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x4c]).unwrap()), "LD C,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x4d]).unwrap()), "LD C,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x4e ,0x45]).unwrap()), "LD C,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x54]).unwrap()), "LD D,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x55]).unwrap()), "LD D,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x5e ,0x45]).unwrap()), "LD E,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x60]).unwrap()), "LD IXH,B");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x61]).unwrap()), "LD IXH,C");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x62]).unwrap()), "LD IXH,D");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x63]).unwrap()), "LD IXH,E");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x64]).unwrap()), "LD IXH,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x65]).unwrap()), "LD IXH,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x66 ,0x45]).unwrap()), "LD H,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x67]).unwrap()), "LD IXH,A");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x68]).unwrap()), "LD IXL,B");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x69]).unwrap()), "LD IXL,C");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x6a]).unwrap()), "LD IXL,D");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x6b]).unwrap()), "LD IXL,E");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x6c]).unwrap()), "LD IXL,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x6d]).unwrap()), "LD IXL,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xdd,  0x6e]).unwrap()), "LD L,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x6F]).unwrap()), "LD IXL,A");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x70 ,0x45]).unwrap()), "LD (IX+NN),B");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x71 ,0x45]).unwrap()), "LD (IX+NN),C");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x72 ,0x45]).unwrap()), "LD (IX+NN),D");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x73 ,0x45]).unwrap()), "LD (IX+NN),E");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x74 ,0x45]).unwrap()), "LD (IX+NN),H");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x75 ,0x45]).unwrap()), "LD (IX+NN),L");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x77 ,0x45]).unwrap()), "LD (IX+NN),A");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x7C]).unwrap()), "LD A,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x7D]).unwrap()), "LD A,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x7E ,0x45]).unwrap()), "LD A,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x84]).unwrap()), "ADD A,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x85]).unwrap()), "ADD A,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x86 ,0x45]).unwrap()), "ADD A,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x8C]).unwrap()), "ADC A,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x8D]).unwrap()), "ADC A,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x8E ,0x45]).unwrap()), "ADC A,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x94]).unwrap()), "SUB A,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x95]).unwrap()), "SUB A,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x96 ,0x45]).unwrap()), "SUB A,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x9C]).unwrap()), "SBC A,IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x9D]).unwrap()), "SBC A,IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0x9E ,0x45]).unwrap()), "SBC A,(IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xA4]).unwrap()), "AND IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xA5]).unwrap()), "AND IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xA6 ,0x45]).unwrap()), "AND (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xAC]).unwrap()), "XOR IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xAD]).unwrap()), "XOR IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xAE ,0x45]).unwrap()), "XOR (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xB4]).unwrap()), "OR IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xB5]).unwrap()), "OR IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xB6 ,0x45]).unwrap()), "OR (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xBC]).unwrap()), "CP IXH");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xBD]).unwrap()), "CP IXL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xBE ,0x45]).unwrap()), "CP (IX+NN)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xE1]).unwrap()), "POP IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xE3]).unwrap()), "EX (SP),IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xE5]).unwrap()), "PUSH IX");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDD ,0xE9]).unwrap()), "JP (IX)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDE ,0x45]).unwrap()), "sbc a, 45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xDF]).unwrap()), "rst $18");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE0]).unwrap()), "ret po");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE1]).unwrap()), "pop hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE2 ,0x34 ,0x12]).unwrap()), "jp po, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE3]).unwrap()), "ex (sp),hL");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE4 ,0x34 ,0x12]).unwrap()), "call po, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE5]).unwrap()), "push hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE6 ,0x45]).unwrap()), "and  45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE7]).unwrap()), "rst $20");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE8]).unwrap()), "ret pe");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xE9]).unwrap()), "jp (hl)");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xEA ,0x34 ,0x12]).unwrap()), "jp pe, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xEB]).unwrap()), "ex de,hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xEC ,0x34 ,0x12]).unwrap()), "call p, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xED]).unwrap()), "ed oPCODEs");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xEE ,0x45]).unwrap()), "xor  45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xEF]).unwrap()), "rst $28");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF0]).unwrap()), "ret p");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF1]).unwrap()), "pop af");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF2 ,0x34 ,0x12]).unwrap()), "jp p, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF3]).unwrap()), "di");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF4 ,0x34 ,0x12]).unwrap()), "call p, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF5]).unwrap()), "push af");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF6 ,0x45]).unwrap()), "or  45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF7]).unwrap()), "rst $30");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF8]).unwrap()), "ret m");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xF9]).unwrap()), "ld sp,hl");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFA ,0x34 ,0x12]).unwrap()), "jp m, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFB]).unwrap()), "ei");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFC ,0x34 ,0x12]).unwrap()), "call m, 1234h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFD]).unwrap()), "fd oPCODEs");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFE ,0x45]).unwrap()), "cp  45h");
        assert_eq!(format!("{}", Instruction::from_bytes(&[0xFF]).unwrap()), "rst $38");
    }
}
