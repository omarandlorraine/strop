#![allow(dead_code)]

use crate::backends::x80::data::InstructionData;
use crate::backends::x80::data::ReadWrite;

pub struct Opcode(pub u8);

enum Condition {
    None,
    Nz, Z, Nc, C, Po, Pe, P, M
}

impl Condition {
    pub const fn zero(&self) -> ReadWrite { ReadWrite::N.read(matches!(self, Self::Nz | Self::Z)) }
    pub const fn carry(&self) -> ReadWrite { ReadWrite::N.read(matches!(self, Self::Nc | Self::C)) }
    pub const fn negative(&self) -> ReadWrite { ReadWrite::N.read(matches!(self, Self::P | Self::M)) }
    pub const fn label(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Nz => "nz",
            Self::Z => "z",
            Self::Nc => "nc",
            Self::C => "c",
            Self::Po => "po",
            Self::Pe => "pe",
            Self::P => "p",
            Self::M => "m",
        }
    }
    const fn cond(sel: u8) -> Self {
        match sel {
            0 => Self::Nz,
            1 => Self::Z,
            2 => Self::Nc,
            3 => Self::C,
            4 => Self::Po,
            5 => Self::Pe,
            6 => Self::P,
            _ => Self::M,
        }
    }
}

enum Alu {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or, 
    Cp,
}

impl Alu {
    const fn alu(sel: u8) -> Self {
        match sel {
            0 => Self::Add,
            1 => Self::Adc,
            2 => Self::Sub,
            3 => Self::Sbc,
            4 => Self::And,
            5 => Self::Xor,
            6 => Self::Or,
            _ => Self::Cp,
        }
    }

    const fn a(&self) -> ReadWrite {
        match self {
            Self::Cp => ReadWrite::R,
            _ => ReadWrite::Rmw,
        }
    }

    const fn carry(&self) -> ReadWrite {
        match self {
            Self::Add => ReadWrite::W,
            Self::Adc => ReadWrite::Rmw,
            Self::Sub => ReadWrite::W,
            Self::Sbc => ReadWrite::Rmw,
            Self::And => ReadWrite::W, // todo: is this right?
            Self::Xor => ReadWrite::W, // todo: is this right?
            Self::Or => ReadWrite::W, // todo: is this right?
            Self::Cp => ReadWrite::W,
        }
    }

    const fn mnemonic(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Adc => "adc",
            Self::Sub => "sub",
            Self::Sbc => "sbc",
            Self::And => "and",
            Self::Xor => "xor",
            Self::Or => "or",
            Self::Cp => "cp",
        }
    }
}

enum Register8 {
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    A,
}

impl Register8 {
    pub const fn r(sel: u8) -> Self {
        match sel {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::M,
            _ => Self::A,
        }
    }
    pub const fn b(&self) -> bool {
        matches!(self, Self::B)
    }
    pub const fn c(&self) -> bool {
        matches!(self, Self::C)
    }
    pub const fn d(&self) -> bool {
        matches!(self, Self::D)
    }
    pub const fn e(&self) -> bool {
        matches!(self, Self::E)
    }
    pub const fn h(&self) -> bool {
        matches!(self, Self::H)
    }
    pub const fn l(&self) -> bool {
        matches!(self, Self::L)
    }
    pub const fn m(&self) -> bool {
        matches!(self, Self::M)
    }
    pub const fn a(&self) -> bool {
        matches!(self, Self::A)
    }
    pub const fn cycle_penalty(&self) -> usize {
        if matches!(self, Self::M) {
            4
        } else {
            0
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::H => "h",
            Self::L => "l",
            Self::M => "(hl)",
            Self::A => "a",
        }
    }
}

enum RegisterPair {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl RegisterPair {
    pub const fn rp(sel: u8) -> Self {
        match sel {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            _ => Self::SP,
        }
    }
    pub const fn rp2(sel: u8) -> Self {
        match Self::rp(sel) {
            Self::SP => Self::AF,
            something_else => something_else,
        }
    }
    pub const fn bc(&self) -> bool {
        matches!(self, Self::BC)
    }
    pub const fn de(&self) -> bool {
        matches!(self, Self::DE)
    }
    pub const fn hl(&self) -> bool {
        matches!(self, Self::HL)
    }
    pub const fn af(&self) -> bool {
        matches!(self, Self::AF)
    }
    pub const fn sp(&self) -> bool {
        matches!(self, Self::SP)
    }
    pub const fn label(&self) -> &'static str {
        match self {
            Self::BC => "bc",
            Self::DE => "de",
            Self::HL => "hl",
            Self::SP => "sp",
            Self::AF => "af",
        }
    }
}

impl Opcode {
    const fn p(&self) -> u8 {
        self.y() >> 1
    }
    const fn q(&self) -> u8 {
        self.y() & 0x01
    }
    const fn x(&self) -> u8 {
        self.0 >> 6 & 0x7
    }
    const fn y(&self) -> u8 {
        self.0 >> 3 & 0x7
    }
    const fn z(&self) -> u8 {
        self.0 & 0x7
    }

    const fn gen_alu_reg(opcode: u8, alu: Alu, reg: Register8) -> Option< InstructionData> {
        Some(InstructionData {
            mnemonic: alu.mnemonic(),
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::W,
            negative: ReadWrite::W,
            half_carry: ReadWrite::W,
            carry: alu.carry(),
            a: alu.a(),
            b: ReadWrite::N.read(reg.b()),
            c: ReadWrite::N.read(reg.c()),
            d: ReadWrite::N.read(reg.d()),
            e: ReadWrite::N.read(reg.e()),
            h: ReadWrite::N.read(reg.h() | reg.m()),
            l: ReadWrite::N.read(reg.l() | reg.m()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [reg.label(), "", ""],
        })
    }

    const fn gen_alu_immediate(opcode: u8, alu: Alu) -> Option< InstructionData> {
        Some(InstructionData {
            mnemonic: alu.mnemonic(),
            flow_control: false,
            opcode,
            bytes: 2,
            cycles: 8,
            zero: ReadWrite::W,
            negative: ReadWrite::W,
            half_carry: ReadWrite::W,
            carry: alu.carry(),
            a: alu.a(),
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
            operands: ["n8", "", ""],
        })
    }

    const fn gen_rst(opcode: u8, rst: u8) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "rst",
            flow_control: true,
            opcode,
            bytes: 1,
            cycles: 16,
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
            operands: [match rst {
                0=> "$00", 1 => "$08", 2 => "$10", 3=> "$18", 4=>"$20", 5=> "$28" , 6=>"$30", 7=>"$38" ,_=> return None,
            }, "", ""],
        })
    }

    const fn gen_jphl(opcode: u8) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "jp",
            flow_control: true,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N,
            b: ReadWrite::N,
            c: ReadWrite::N,
            d: ReadWrite::N,
            e: ReadWrite::N,
            h: ReadWrite::R,
            l: ReadWrite::R,
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: ["(hl)","",""],
        })
    }

    const fn gen_nop() -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "nop",
            flow_control: false,
            opcode: 0x0,
            bytes: 1,
            cycles: 4,
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
            operands: ["", "", ""],
        })
    }

    const fn gen_interrupt_enable(opcode: u8, value: bool) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: if value { "ei" } else { "di" },
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
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
            operands: ["", "", ""],
        })
    }

    const fn gen_carry_flag(opcode: u8, value: bool) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: if value { "scf" } else { "ccf" },
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::W,
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
            operands: ["", "", ""],
        })
    }

    const fn gen_misc(opcode: u8, mnemonic: &'static str) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic,
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::Rmw,
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
            operands: ["", "", ""],
        })
    }

    const fn gen_rmw_a(opcode: u8, mnemonic: &'static str) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic,
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::W,
            negative: ReadWrite::W,
            half_carry: ReadWrite::W,
            carry: ReadWrite::W,
            a: ReadWrite::Rmw,
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
            operands: ["", "", ""],
        })
    }
    const fn gen_halt() -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "halt",
            flow_control: false,
            opcode: 0x76,
            bytes: 1,
            cycles: 4,
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
            operands: ["", "", ""],
        })
    }

    const fn gen_inc8_or_dec8(
        opcode: u8,
        mnemonic: &'static str,
        p: u8,
    ) -> Option<InstructionData> {
        let operand = Register8::r(p);

        Some(InstructionData {
            mnemonic,
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4 + operand.cycle_penalty(),
            zero: ReadWrite::W,
            negative: ReadWrite::W,
            half_carry: ReadWrite::W,
            carry: ReadWrite::N,
            a: ReadWrite::N.read(operand.a()).write(operand.a()),
            b: ReadWrite::N.read(operand.b()).write(operand.b()),
            c: ReadWrite::N.read(operand.c()).write(operand.c()),
            d: ReadWrite::N.read(operand.d()).write(operand.d()),
            e: ReadWrite::N.read(operand.e()).write(operand.e()),
            h: ReadWrite::N
                .read(operand.h() | operand.m())
                .write(operand.h()),
            l: ReadWrite::N
                .read(operand.l() | operand.m())
                .write(operand.l()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [operand.label(), "", ""],
        })
    }

    const fn gen_inc16_or_dec16(opcode: u8, q: u8, p: u8) -> Option<InstructionData> {
        let mnemonic = if q == 0 { "inc" } else { "dec" };
        let operand = RegisterPair::rp(p);

        Some(InstructionData {
            mnemonic,
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 8,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N,
            b: ReadWrite::N.read(operand.bc()).write(operand.bc()),
            c: ReadWrite::N.read(operand.bc()).write(operand.bc()),
            d: ReadWrite::N.read(operand.de()).write(operand.de()),
            e: ReadWrite::N.read(operand.de()).write(operand.de()),
            h: ReadWrite::N.read(operand.hl()).write(operand.hl()),
            l: ReadWrite::N.read(operand.hl()).write(operand.hl()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N.read(operand.sp()).write(operand.sp()),
            i: ReadWrite::N,
            operands: [operand.label(), "", ""],
        })
    }

    const fn gen_ex_de_hl(opcode: u8) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "ex",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 16,
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
            operands: ["de", "hl", ""],
        })
    }

    const fn gen_indirect_store(opcode: u8, reg: u8) -> Option<InstructionData> {
        let load_hl = reg == 2;
        let load_a = !load_hl;
        let ptr_bc = reg == 0;
        let ptr_de = reg == 1;
        let ptr_nn = !(ptr_bc || ptr_de);

        let src = if ptr_bc {
            "(bc)"
        } else if ptr_de {
            "(de)"
        } else {
            "(a16)"
        };

        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: if ptr_nn { 3 } else { 1 },
            cycles: 8,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N.write(load_a),
            b: ReadWrite::N.read(ptr_bc),
            c: ReadWrite::N.read(ptr_bc),
            d: ReadWrite::N.read(ptr_de),
            e: ReadWrite::N.read(ptr_de),
            h: ReadWrite::N.write(load_hl),
            l: ReadWrite::N.write(load_hl),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [if load_a { "a" } else { "hl" }, src, ""],
        })
    }

    const fn gen_indirect_load(opcode: u8, reg: u8) -> Option<InstructionData> {
        let load_hl = reg == 2;
        let load_a = !load_hl;
        let ptr_bc = reg == 0;
        let ptr_de = reg == 1;
        let ptr_nn = !(ptr_bc || ptr_de);

        let src = if ptr_bc {
            "(bc)"
        } else if ptr_de {
            "(de)"
        } else {
            "(a16)"
        };

        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: if ptr_nn { 3 } else { 1 },
            cycles: 8,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N.read(load_a),
            b: ReadWrite::N.read(ptr_bc),
            c: ReadWrite::N.read(ptr_bc),
            d: ReadWrite::N.read(ptr_de),
            e: ReadWrite::N.read(ptr_de),
            h: ReadWrite::N.read(load_hl),
            l: ReadWrite::N.read(load_hl),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [src, if load_a { "a" } else { "hl" }, ""],
        })
    }

    const fn gen_add16(opcode: u8, reg: u8) -> Option<InstructionData> {
        let operand = RegisterPair::rp(reg);
        Some(InstructionData {
            mnemonic: "add",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 8,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::W,
            a: ReadWrite::N,
            b: ReadWrite::N.read(operand.bc()),
            c: ReadWrite::N.read(operand.bc()),
            d: ReadWrite::N.read(operand.de()),
            e: ReadWrite::N.read(operand.de()),
            h: ReadWrite::Rmw,
            l: ReadWrite::Rmw,
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N.read(operand.sp()),
            i: ReadWrite::N,
            operands: ["hl", operand.label(), ""],
        })
    }

    const fn gen_immediate_load8(opcode: u8, reg: u8) -> Option<InstructionData> {
        let dest = Register8::r(reg);
        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: 2,
            cycles: 8 + dest.cycle_penalty(),
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N.write(dest.a()),
            b: ReadWrite::N.write(dest.b()),
            c: ReadWrite::N.write(dest.c()),
            d: ReadWrite::N.write(dest.d()),
            e: ReadWrite::N.write(dest.e()),
            h: ReadWrite::N.write(dest.h()).read(dest.m()),
            l: ReadWrite::N.write(dest.l()).read(dest.m()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [dest.label(), "n8", ""],
        })
    }

    const fn gen_immediate_load16(opcode: u8, reg: u8) -> Option<InstructionData> {
        let dest = RegisterPair::rp(reg);
        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: 3,
            cycles: 12,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N,
            b: ReadWrite::N.write(dest.bc()),
            c: ReadWrite::N.write(dest.bc()),
            d: ReadWrite::N.write(dest.de()),
            e: ReadWrite::N.write(dest.de()),
            h: ReadWrite::N.write(dest.hl()),
            l: ReadWrite::N.write(dest.hl()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N.write(dest.sp()),
            i: ReadWrite::N,
            operands: [dest.label(), "n16", ""],
        })
    }

    const fn gen_ret(
        opcode: u8,
        cond: Condition
    ) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "ret",
            flow_control: true,
            opcode,
            bytes: 1,
            cycles: 20,
            zero: cond.zero(),
            negative: cond.negative(),
            half_carry: ReadWrite::N,
            carry: cond.carry(),
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
            operands: [cond.label(), "", ""],
        })
    }

    const fn gen_register_to_register_load(
        opcode: u8,
        src: u8,
        dst: u8,
    ) -> Option<InstructionData> {
        let src = Register8::r(src);
        let dst = Register8::r(dst);
        let m = src.m() | dst.m();
        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: if m {8 } else {4},
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: ReadWrite::N.read(src.a()).write(dst.a()),
            b: ReadWrite::N.read(src.b()).write(dst.b()),
            c: ReadWrite::N.read(src.c()).write(dst.c()),
            d: ReadWrite::N.read(src.d()).write(dst.d()),
            e: ReadWrite::N.read(src.e()).write(dst.e()),
            h: ReadWrite::N.read(src.h()|m).write(dst.h()),
            l: ReadWrite::N.read(src.l()|m).write(dst.l()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [dst.label(), src.label(), ""],
        })
    }

    const fn gen_call(
        opcode: u8,
        cond: Condition,
    ) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "call",
            flow_control: true,
            opcode,
            bytes: 3,
            cycles: 24,
            zero: cond.zero(),
            negative: cond.negative(),
            half_carry: ReadWrite::N,
            carry: cond.carry(),
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
            operands: if matches!(cond , Condition::None) {
                ["a16", "", ""]
            } else {
                [cond.label(), "a16", ""]
            }
        })
    }

    const fn gen_jp_absolute(
        opcode: u8,
        cond: Condition,
    ) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "jp",
            flow_control: true,
            opcode,
            bytes: 3,
            cycles: 16,
            zero: cond.zero(),
            negative: cond.negative(),
            half_carry: ReadWrite::N,
            carry: cond.carry(),
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
            operands: if matches!(cond , Condition::None) {
                ["a16", "", ""]
            } else {
                [cond.label(), "a16", ""]
            }
        })
    }

    const fn gen_push(
        opcode: u8,
        dst: RegisterPair,
    ) -> Option<InstructionData> {
        let flags = if matches!(dst, RegisterPair::AF) {
            ReadWrite::R
        } else {
            ReadWrite::N
        };
        Some(InstructionData {
            mnemonic: "push",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 16,
            zero: flags,
            negative: flags,
            half_carry: flags,
            carry: flags,
            a: flags,
            b: ReadWrite::N.read(dst.bc()),
            c: ReadWrite::N.read(dst.bc()),
            d: ReadWrite::N.read(dst.de()),
            e: ReadWrite::N.read(dst.de()),
            h: ReadWrite::N.read(dst.hl()),
            l: ReadWrite::N.read(dst.hl()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [dst.label(), "", ""],
        })
    }

    const fn gen_xthl(
        opcode: u8,
    ) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "ex",
            flow_control: false,
            opcode,
            bytes: 1,
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
            h: ReadWrite::Rmw,
            l: ReadWrite::Rmw,
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::R,
            i: ReadWrite::N,
            operands: ["(sp)", "hl", ""],
        })
    }

    const fn gen_sphl(
        opcode: u8,
    ) -> Option<InstructionData> {
        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: 1,
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
            h: ReadWrite::R,
            l: ReadWrite::R,
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::W,
            i: ReadWrite::N,
            operands: ["sp", "hl", ""],
        })
    }

    const fn gen_pop(
        opcode: u8,
        dst: RegisterPair,
    ) -> Option<InstructionData> {
        let flags = if matches!(dst, RegisterPair::AF) {
            ReadWrite::W
        } else {
            ReadWrite::N
        };
        Some(InstructionData {
            mnemonic: "pop",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 12,
            zero: flags,
            negative: flags,
            half_carry: flags,
            carry: flags,
            a: flags,
            b: ReadWrite::N.write(dst.bc()),
            c: ReadWrite::N.write(dst.bc()),
            d: ReadWrite::N.write(dst.de()),
            e: ReadWrite::N.write(dst.de()),
            h: ReadWrite::N.write(dst.hl()),
            l: ReadWrite::N.write(dst.hl()),
            r: ReadWrite::N,
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: [dst.label(), "", ""],
        })
    }

    pub const fn parse(&self) -> Option<InstructionData> {
        match (self.x(), self.y(), self.z()) {
            (0, 0, 0) => Self::gen_nop(),
            (0, _, 1) if self.q() == 0 => Self::gen_immediate_load16(self.0, self.p()),
            (0, _, 1) if self.q() == 1 => Self::gen_add16(self.0, self.p()),
            (0, _, 2) if self.q() == 0 => Self::gen_indirect_load(self.0, self.p()),
            (0, _, 2) if self.q() == 1 => Self::gen_indirect_store(self.0, self.p()),
            (0, _, 3) => Self::gen_inc16_or_dec16(self.0, self.q(), self.p()),
            (0, y, 4) => Self::gen_inc8_or_dec8(self.0, "inc", y),
            (0, y, 5) => Self::gen_inc8_or_dec8(self.0, "dec", y),
            (0, y, 6) => Self::gen_immediate_load8(self.0, y),
            (0, 0, 7) => Self::gen_rmw_a(self.0, "rlca"),
            (0, 1, 7) => Self::gen_rmw_a(self.0, "rrca"),
            (0, 2, 7) => Self::gen_rmw_a(self.0, "rla"),
            (0, 3, 7) => Self::gen_rmw_a(self.0, "rra"),
            (0, 4, 7) => Self::gen_rmw_a(self.0, "daa"),
            (0, 5, 7) => Self::gen_misc(self.0, "cpl"),
            (0, _, 7) => Self::gen_carry_flag(self.0, self.q() == 0),

            (1, 6, 6) => Self::gen_halt(),
            (1, src, dst) => Self::gen_register_to_register_load(self.0, dst, src),

            (2, alu, reg) => Self::gen_alu_reg(self.0, Alu::alu(alu), Register8::r(reg)),

            (3, cond, 0) => Self::gen_ret(self.0, Condition::cond(cond)),
            (3, _, 1) if self.q() == 0 => Self::gen_pop(self.0, RegisterPair::rp2(self.p())),
            (3, 1, 1) => Self::gen_ret(self.0, Condition::None),
            (3, 5, 1) => Self::gen_jphl(self.0),
            (3, cond, 2) => Self::gen_jp_absolute(self.0,  Condition::cond(cond)),
            (3, 0, 3) =>Self::gen_jp_absolute(self.0,  Condition::None),
            (3, 4, 3) => Self::gen_xthl(self.0),
            (3, 5, 3) => Self::gen_ex_de_hl(self.0),
            (3, 6, 3) => Self::gen_interrupt_enable(self.0, self.q() != 0),
            (3, 7, 3) => Self::gen_interrupt_enable(self.0, self.q() != 0),
            (3, cond, 4) => Self::gen_call(self.0,  Condition::cond(cond)),
            (3, _, 5) if self.q() == 0 => Self::gen_push(self.0,  RegisterPair::rp2(self.p())),
            (3, 1, 5) => Self::gen_call(self.0,  Condition::None),
            (3, alu, 6) => Self::gen_alu_immediate(self.0,  Alu::alu(alu)),
            (3, rst, 7) => Self::gen_rst(self.0, rst),

            _ => None,
        }
    }
}
