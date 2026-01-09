#![allow(dead_code)]

use crate::backends::x80::data::InstructionData;
use crate::backends::x80::data::ReadWrite;

pub struct Opcode(u8);

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
            "(nn)"
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
            b: ReadWrite::N.read(dest.bc()),
            c: ReadWrite::N.read(dest.bc()),
            d: ReadWrite::N.read(dest.de()),
            e: ReadWrite::N.read(dest.de()),
            h: ReadWrite::W.read(dest.hl()),
            l: ReadWrite::W.read(dest.hl()),
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

    const fn gen_register_to_register_load(
        opcode: u8,
        src: u8,
        dst: u8,
    ) -> Option<InstructionData> {
        const fn hl(facit: u8, src: u8, dst: u8) -> ReadWrite {
            let horl = (src < 7 && src > 3) || (dst < 7 && dst > 3);
            match (rmw(facit, src, dst), horl) {
                (ReadWrite::N, false) => ReadWrite::N,
                (ReadWrite::N, true) => ReadWrite::R,
                (ReadWrite::R, _) => ReadWrite::R,
                (ReadWrite::W, false) => ReadWrite::W,
                (ReadWrite::W, true) => ReadWrite::Rmw,
                (ReadWrite::Rmw, _) => ReadWrite::Rmw,
            }
        }
        const fn rmw(facit: u8, src: u8, dst: u8) -> ReadWrite {
            match (facit == src, facit == dst) {
                (false, false) => ReadWrite::N,
                (false, true) => ReadWrite::W,
                (true, false) => ReadWrite::R,
                (true, true) => ReadWrite::Rmw,
            }
        }
        Some(InstructionData {
            mnemonic: "ld",
            flow_control: false,
            opcode,
            bytes: 1,
            cycles: 4,
            zero: ReadWrite::N,
            negative: ReadWrite::N,
            half_carry: ReadWrite::N,
            carry: ReadWrite::N,
            a: rmw(7, src, dst),
            b: rmw(0, src, dst),
            c: rmw(1, src, dst),
            d: rmw(2, src, dst),
            e: rmw(3, src, dst),
            h: hl(4, src, dst),
            l: hl(5, src, dst),
            r: rmw(6, src, dst),
            ixh: ReadWrite::N,
            ixl: ReadWrite::N,
            iyh: ReadWrite::N,
            iyl: ReadWrite::N,
            sp: ReadWrite::N,
            i: ReadWrite::N,
            operands: ["", "", ""],
        })
    }

    pub const fn parse(&self) -> Option<InstructionData> {
        match (self.x(), self.y(), self.z()) {
            (0, 0, 0) => Self::gen_nop(),
            (0, _, 1) if self.q() == 0 => Self::gen_immediate_load16(self.0, self.p()),
            (0, _, 1) => Self::gen_add16(self.0, self.p()),
            (0, _, 2) if self.q() == 0 => Self::gen_indirect_load(self.0, self.p()),
            (0, _, 3) => Self::gen_inc16_or_dec16(self.0, self.q(), self.p()),
            (1, 6, 6) => Self::gen_halt(),
            (1, src, dst) => Self::gen_register_to_register_load(self.0, src, dst),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn n() {
        use crate::backends::x80::parse::Opcode;
        for op in 0u8..=255 {
            let old = &crate::backends::i8080::data::UNPREFIXED[op as usize];
            if old.is_none() {
                continue;
            }
            assert_eq!(Opcode(op).parse(), *old);
        }
    }
}
