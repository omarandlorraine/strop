//! The `stm8` backend, for generating code sequences for the 8-bit microcontroller family by
//! STMicroelectronics.

// Just temporarily allow these lints while we're working on a half-baked implementation
#![allow(missing_debug_implementations, missing_docs, dead_code)]

use crate::machine::Instruction;
use crate::machine::Strop;
use rand::random;
use randomly::randomly;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operand8 {
    A,
    Xl,
    Yl,
    Xh,
    Yh,
    Imm8(u8),
    Abs(u16),
}

#[derive(Clone, Copy)]
pub enum Operand16 {
    Imm(u16),
    Abs(u16),
}

#[derive(Clone, Copy)]
pub enum Register16 {
    X,
    Y,
}

impl Register16 {
    fn get_u16(&self, s: &Stm8) -> Option<u16> {
        match self {
            Register16::X => s.x.get_u16(),
            Register16::Y => s.y.get_u16(),
        }
    }

    fn set_u16(&self, s: &mut Stm8, v: Option<u16>) {
        match self {
            Register16::X => s.x.set_u16(v),
            Register16::Y => s.y.set_u16(v),
        }
    }

    fn set_i16(&self, s: &mut Stm8, v: Option<i16>) {
        match self {
            Register16::X => s.x.set_i16(v),
            Register16::Y => s.y.set_i16(v),
        }
    }
}

impl fmt::Display for Register16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register16::X => write!(f, "x"),
            Register16::Y => write!(f, "y"),
        }
    }
}

impl Strop for u8 {
    fn random() -> u8 {
        random()
    }

    fn mutate(&mut self) {
        randomly!(
            /* could try incrementing or decrementing */
            { *self = self.wrapping_add(1) }
            { *self = self.wrapping_sub(1) }

            /* could try flipping a bit */
            {
                let bit_select = 1_u8.rotate_left(rand::thread_rng().gen_range(0..8));
                *self ^= bit_select;
            }
        );
    }
}

impl Strop for u16 {
    fn random() -> u16 {
        random()
    }

    fn mutate(&mut self) {
        randomly!(
            /* could try incrementing or decrementing */
            { *self = self.wrapping_add(1) }
            { *self = self.wrapping_sub(1) }

            /* could try flipping a bit */
            {
                let bit_select = 1_u16.rotate_left(rand::thread_rng().gen_range(0..16));
                *self ^= bit_select;
            }
        );
    }
}

impl Strop for Register16 {
    fn random() -> Register16 {
        use Register16::*;
        randomly!(
            {X}
            {Y}
        )
    }
    fn mutate(&mut self) {
        *self = Register16::random();
    }
}

impl Strop for Operand8 {
    fn random() -> Operand8 {
        use Operand8::*;
        randomly!(
            {Imm8(random())}
            {Abs(random())}
        )
    }
    fn mutate(&mut self) {
        use Operand8::*;
        let e = match self {
            A => A,
            Xl => Xl,
            Yl => Yl,
            Xh => Xh,
            Yh => Yh,
            Imm8(v) => {
                let e = v;
                e.mutate();
                Imm8(*e)
            }
            Abs(addr) => Abs(*addr),
        };
        *self = e;
    }
}

impl Strop for Operand16 {
    fn random() -> Operand16 {
        use Operand16::*;
        randomly!(
            {Imm(random())}
            {Abs(random())}
        )
    }
    fn mutate(&mut self) {
        use Operand16::*;
        let e = match self {
            Imm(v) => {
                let e = v;
                e.mutate();
                Imm(*e)
            }
            Abs(addr) => Abs(*addr),
        };
        *self = e;
    }
}

impl Operand16 {
    fn get_u16(self, s: &Stm8) -> Option<u16> {
        use Operand16::*;
        match self {
            Imm(x) => Some(x),
            Abs(addr) => {
                let low = s.read_mem(Some(addr));
                let high = s.read_mem(Some(addr + 1));
                low.zip(high).map(|(l, h)| u16::from_le_bytes([l, h]))
            }
        }
    }
}

impl fmt::Display for Operand16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Operand16::*;
        match self {
            Imm(x) => write!(f, "#${:04x}", x),
            Abs(addr) => write!(f, "${:04x}", addr),
        }
    }
}

impl Operand8 {
    fn set_u8(self, s: &mut Stm8, val: Option<u8>) {
        use Operand8::*;
        match self {
            A => s.a = val,
            Xl => s.x.low = val,
            Yl => s.y.low = val,
            Xh => s.x.high = val,
            Yh => s.y.high = val,
            Imm8(_) => panic!(),
            Abs(addr) => s.write_mem(addr, val),
        }
    }

    fn get_u8(self, s: &Stm8) -> Option<u8> {
        use Operand8::*;
        match self {
            A => s.a,
            Xl => s.x.low,
            Yl => s.y.low,
            Xh => s.x.high,
            Yh => s.y.high,
            Imm8(x) => Some(x),
            Abs(addr) => s.read_mem(Some(addr)),
        }
    }
}

#[derive(Default)]
pub struct IndexRegister {
    high: Option<u8>,
    low: Option<u8>,
}

impl IndexRegister {
    fn set_u16(&mut self, val: Option<u16>) {
        let v = val.map(|v| v.to_be_bytes());
        self.high = v.map(|v| u8::from_ne_bytes(v[0].to_ne_bytes()));
        self.low = v.map(|v| u8::from_ne_bytes(v[1].to_ne_bytes()));
    }

    fn set_i16(&mut self, val: Option<i16>) {
        let v = val.map(|v| v.to_be_bytes());
        self.high = v.map(|v| u8::from_ne_bytes(v[0].to_ne_bytes()));
        self.low = v.map(|v| u8::from_ne_bytes(v[1].to_ne_bytes()));
    }

    fn get_u16(&self) -> Option<u16> {
        self.low
            .zip(self.high)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }
}

#[derive(Default)]
pub struct Stm8 {
    a: Option<u8>,
    x: IndexRegister,
    y: IndexRegister,
    m: HashMap<u16, Option<u8>>,
    carry: Option<bool>,
    halfcarry: Option<bool>,
    overflow: Option<bool>,
    zero: Option<bool>,
    sign: Option<bool>,
}

impl Stm8 {
    fn get_register16(&self, select: Register16) -> Option<u16> {
        match select {
            Register16::X => self.x.get_u16(),
            Register16::Y => self.y.get_u16(),
        }
    }

    fn set_register16(&mut self, select: Register16, val: Option<u16>) {
        match select {
            Register16::X => self.x.set_u16(val),
            Register16::Y => self.y.set_u16(val),
        }
    }

    fn read_mem(&self, addr: Option<u16>) -> Option<u8> {
        if let Some(addr) = addr {
            self.m[&addr]
        } else {
            None
        }
    }

    fn write_mem(&mut self, addr: u16, v: Option<u8>) {
        self.m.insert(addr, v);
    }
}

#[derive(Clone, Copy)]
pub enum Stm8Operands {
    None,
    Ld(Operand8, Operand8),
    Mov(Operand8, Operand8),
    Rmw(Operand8),
    Alu8(Operand8),
    Alu16(Register16, Operand16),
    Bits(u16, usize),
    R16(Register16),
    Exg(Operand8),
}

impl Stm8Operands {
    fn get_rmw(self) -> Operand8 {
        match self {
            Stm8Operands::Rmw(operand) => operand,
            _ => panic!(),
        }
    }

    fn get_ld(self) -> (Operand8, Operand8) {
        match self {
            Stm8Operands::Ld(src, dst) => (src, dst),
            Stm8Operands::Mov(src, dst) => (src, dst),
            _ => panic!(),
        }
    }

    fn get_alu8(self) -> Operand8 {
        match self {
            Stm8Operands::Alu8(operand) => operand,
            Stm8Operands::Exg(operand) => operand,
            _ => panic!(),
        }
    }

    fn get_r16(self) -> Register16 {
        match self {
            Stm8Operands::R16(r) => r,
            _ => panic!(),
        }
    }

    fn get_alu16(self) -> (Register16, Operand16) {
        match self {
            Stm8Operands::Alu16(r, operand) => (r, operand),
            _ => panic!(),
        }
    }

    fn get_bits(self) -> (u16, usize) {
        match self {
            Stm8Operands::Bits(addr, bit) => (addr, bit),
            _ => panic!(),
        }
    }
}

impl Strop for Stm8Operands {
    fn random() -> Self {
        unimplemented!();
    }

    fn mutate(&mut self) {
        use rand::prelude::SliceRandom;
        use Operand8::*;

        match self {
            Stm8Operands::None => (),
            Stm8Operands::Mov(src, dst) => {
                randomly!(
                    { *src = Imm8(random()) }
                    { *src = Abs(random()) }
                    { src.mutate() }
                    { dst.mutate() })
            }
            Stm8Operands::Ld(src, dst) => {
                if *src == Operand8::A {
                    randomly!(
                        { dst.mutate() }
                        {*dst = *vec!(Xl, Xh, Yl, Yh, A).choose(&mut rand::thread_rng()).unwrap() }
                    );
                } else {
                    randomly!(
                        { src.mutate() }
                        {*src = *vec!(Xl, Xh, Yl, Yh, A).choose(&mut rand::thread_rng()).unwrap() }
                    );
                }
            }
            Stm8Operands::R16(x) => x.mutate(),
            Stm8Operands::Rmw(v) => v.mutate(),
            Stm8Operands::Alu8(v) => v.mutate(),
            Stm8Operands::Alu16(r, v) => randomly!({r.mutate()} {v.mutate()}),
            Stm8Operands::Bits(addr, bit) => randomly!(
                {addr.mutate()}
                {*bit = (*bit + 1) % 8}
                {*bit = (*bit + 7) % 8}
            ),
            Stm8Operands::Exg(v) => {
                randomly!(
                    {*v = Operand8::Xl}
                    {*v = Operand8::Yl}
                    {*v = Operand8::Abs(random())}
                    {v.mutate()}
                );
            }
        }
    }
}

/// Represents a STM8 Instruction
#[derive(Clone, Copy)]
pub struct Stm8Instruction {
    mnem: &'static str,
    disassemble: fn(&Stm8Instruction, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Stm8Instruction, &mut Stm8),
    operand: Stm8Operands,
}

fn disassemble(insn: &Stm8Instruction, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match insn.operand {
        Stm8Operands::None => {
            write!(f, "\t{}", insn.mnem)
        }
        Stm8Operands::Rmw(v) => {
            write!(f, "\t{} {:?}", insn.mnem, v)
        }
        Stm8Operands::Alu8(v) => {
            write!(f, "\t{} a, {:?}", insn.mnem, v)
        }
        Stm8Operands::R16(r) => {
            write!(f, "\t{} {}", insn.mnem, r)
        }
        Stm8Operands::Exg(v) => {
            write!(f, "\t{} a, {}", insn.mnem, v)
        }
        Stm8Operands::Alu16(r, v) => {
            write!(f, "\t{} {}, {}", insn.mnem, r, v)
        }
        Stm8Operands::Ld(r, v) => {
            write!(f, "\t{} {}, {}", insn.mnem, r, v)
        }
        Stm8Operands::Mov(r, v) => {
            write!(f, "\t{} {}, {}", insn.mnem, r, v)
        }
        Stm8Operands::Bits(addr, bit) => {
            write!(f, "\t{} ${:04x}, #{}", insn.mnem, addr, bit)
        }
    }
}

const ADC: Stm8Instruction = Stm8Instruction {
    mnem: "adc",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a
            .zip(m)
            .zip(s.carry)
            .map(|((a, m), c)| a.wrapping_add(m).wrapping_add(if c { 1 } else { 0 }));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.halfcarry = carrytests.map(|t| t & 0x08 != 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const ADD: Stm8Instruction = Stm8Instruction {
    mnem: "add",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a.wrapping_add(m));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.halfcarry = carrytests.map(|t| t & 0x08 != 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const ADDW: Stm8Instruction = Stm8Instruction {
    mnem: "addw",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let (dst, src) = insn.operand.get_alu16();
        let m = src.get_u16(s).map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
        let a = dst.get_u16(s).map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a.wrapping_add(m));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.halfcarry = carrytests.map(|t| t & 0x0080 != 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
        dst.set_i16(s, r);
    },
};

const AND: Stm8Instruction = Stm8Instruction {
    mnem: "and",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const BCCM: Stm8Instruction = Stm8Instruction {
    mnem: "bccm",
    disassemble,
    operand: Stm8Operands::Bits(0, 0),
    handler: |insn, s| {
        let (addr, bit) = insn.operand.get_bits();
        let m = s.read_mem(Some(addr));
        let r = m.zip(s.carry).map(|(a, c)| {
            if c {
                a | (0x01 << bit)
            } else {
                a & !(0x01 << bit)
            }
        });
        s.write_mem(addr, r);
    },
};

const BCP: Stm8Instruction = Stm8Instruction {
    mnem: "bcp",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
    },
};

const BCPL: Stm8Instruction = Stm8Instruction {
    mnem: "bcpl",
    disassemble,
    operand: Stm8Operands::Bits(0, 0),
    handler: |insn, s| {
        let (addr, bit) = insn.operand.get_bits();
        let m = s.read_mem(Some(addr));
        let r = m.map(|m| m ^ (0x01 << bit));
        s.write_mem(addr, r);
    },
};

const BRES: Stm8Instruction = Stm8Instruction {
    mnem: "bres",
    disassemble,
    operand: Stm8Operands::Bits(0, 0),
    handler: |insn, s| {
        let (addr, bit) = insn.operand.get_bits();
        let m = s.read_mem(Some(addr));
        let r = m.map(|m| m & !(0x01 << bit));
        s.write_mem(addr, r);
    },
};

const BSET: Stm8Instruction = Stm8Instruction {
    mnem: "bset",
    disassemble,
    operand: Stm8Operands::Bits(0, 0),
    handler: |insn, s| {
        let (addr, bit) = insn.operand.get_bits();
        let m = s.read_mem(Some(addr));
        let r = m.map(|m| m | (0x01 << bit));
        s.write_mem(addr, r);
    },
};

const CCF: Stm8Instruction = Stm8Instruction {
    mnem: "ccf",
    disassemble,
    operand: Stm8Operands::None,
    handler: |_insn, s| {
        s.carry = s.carry.map(|c| !c);
    },
};

const CLR: Stm8Instruction = Stm8Instruction {
    mnem: "clr",
    disassemble,
    operand: Stm8Operands::Rmw(Operand8::A),
    handler: |insn, s| {
        insn.operand.get_rmw().set_u8(s, Some(0));
        s.zero = Some(true);
        s.sign = Some(false);
    },
};

const CLRW: Stm8Instruction = Stm8Instruction {
    mnem: "clrw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        insn.operand.get_r16().set_u16(s, Some(0));
        s.zero = Some(true);
        s.sign = Some(false);
    },
};

const CP: Stm8Instruction = Stm8Instruction {
    mnem: "cp",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a.wrapping_sub(m));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
    },
};

const CPW: Stm8Instruction = Stm8Instruction {
    mnem: "cpw",
    disassemble,
    operand: Stm8Operands::Alu16(Register16::X, Operand16::Imm(0)),
    handler: |insn, s| {
        let (dst, src) = insn.operand.get_alu16();
        let m = src.get_u16(s).map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
        let a = dst.get_u16(s).map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a.wrapping_sub(m));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
    },
};

const CPL: Stm8Instruction = Stm8Instruction {
    mnem: "cpl",
    disassemble,
    operand: Stm8Operands::Rmw(Operand8::A),
    handler: |insn, s| {
        let val = insn.operand.get_rmw().get_u8(s);
        let r = val.map(|a| a ^ 0xff);
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_rmw().set_u8(s, r);
    },
};

const CPLW: Stm8Instruction = Stm8Instruction {
    mnem: "cplw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        let val = insn.operand.get_r16();
        let a = val.get_u16(s);
        let r = a.map(|a| a ^ 0xffff);
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_r16().set_u16(s, r);
    },
};

const DEC: Stm8Instruction = Stm8Instruction {
    mnem: "dec",
    disassemble,
    operand: Stm8Operands::Rmw(Operand8::A),
    handler: |insn, s| {
        let val = insn.operand.get_rmw().get_u8(s);
        let r = val.map(|a| a.wrapping_sub(1));
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_rmw().set_u8(s, r);
    },
};

const DECW: Stm8Instruction = Stm8Instruction {
    mnem: "decw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        let val = insn.operand.get_r16();
        let a = val.get_u16(s);
        let r = a.map(|a| a.wrapping_sub(1));
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_r16().set_u16(s, r);
    },
};

const DIV: Stm8Instruction = Stm8Instruction {
    mnem: "div",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        use std::convert::TryInto;
        let dividend = insn.operand.get_r16().get_u16(s);
        let divisor = s.a;

        if dividend.is_none() || divisor.is_none() {
            s.a = None;
            insn.operand.get_r16().set_u16(s, None);
            s.carry = None;
            return;
        }
        if divisor.unwrap() != 0 {
            let quotient: u16 = dividend.unwrap() / (divisor.unwrap() as u16);
            let remainder: u8 = (dividend.unwrap() % (divisor.unwrap() as u16))
                .try_into()
                .unwrap();
            s.a = Some(remainder);
            insn.operand.get_r16().set_u16(s, Some(quotient));
            s.carry = Some(false);
            s.zero = Some(quotient == 0);
        } else {
            // division by zero; the quotient and remainder are not written to the registers.
            s.carry = Some(true);
        }
    },
};

const DIVW: Stm8Instruction = Stm8Instruction {
    mnem: "divw x, y",
    disassemble,
    operand: Stm8Operands::None,
    handler: |_insn, s| {
        let dividend = s.get_register16(Register16::X);
        let divisor = s.get_register16(Register16::Y);

        if dividend.is_none() || divisor.is_none() {
            s.set_register16(Register16::X, None);
            s.set_register16(Register16::Y, None);
            s.carry = None;
            return;
        }
        if divisor.unwrap() == 0 {
            // division by zero; the quotient and remainder are indeterminate
            s.set_register16(Register16::X, None);
            s.set_register16(Register16::Y, None);
            s.carry = Some(false);
            s.zero = None;
        } else {
            let quotient: u16 = dividend.unwrap() / divisor.unwrap();
            let remainder: u16 = dividend.unwrap() % divisor.unwrap();
            s.set_register16(Register16::X, Some(quotient));
            s.set_register16(Register16::Y, Some(remainder));
            s.zero = Some(quotient == 0);
            s.carry = Some(true);
        }
    },
};

const EXG: Stm8Instruction = Stm8Instruction {
    mnem: "exg",
    disassemble,
    operand: Stm8Operands::Exg(Operand8::Xl),
    handler: |insn, s| {
        let src = insn.operand.get_alu8().get_u8(s);
        insn.operand.get_alu8().set_u8(s, s.a);
        s.a = src;
    },
};

const EXGW: Stm8Instruction = Stm8Instruction {
    mnem: "exgw",
    disassemble,
    operand: Stm8Operands::Exg(Operand8::Xl),
    handler: |_insn, s| {
        let src = s.x.get_u16();
        s.x.set_u16(s.y.get_u16());
        s.y.set_u16(src);
    },
};

const INC: Stm8Instruction = Stm8Instruction {
    mnem: "inc",
    disassemble,
    operand: Stm8Operands::Rmw(Operand8::A),
    handler: |insn, s| {
        let val = insn.operand.get_rmw().get_u8(s);
        let r = val.map(|a| a.wrapping_add(1));
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_rmw().set_u8(s, r);
    },
};

const INCW: Stm8Instruction = Stm8Instruction {
    mnem: "incw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        let val = insn.operand.get_r16();
        let a = val.get_u16(s);
        let r = a.map(|a| a.wrapping_add(1));
        s.carry = Some(true);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_r16().set_u16(s, r);
    },
};

const LD: Stm8Instruction = Stm8Instruction {
    mnem: "ld",
    disassemble,
    operand: Stm8Operands::Ld(Operand8::A, Operand8::Imm8(0)),
    handler: |insn, s| {
        let (src, dst) = insn.operand.get_ld();
        s.zero = src.get_u8(s).map(|r| r == 0);
        s.sign = src.get_u8(s).map(|r| r.leading_zeros() == 0);
        dst.set_u8(s, src.get_u8(s));
    },
};

const LDW: Stm8Instruction = Stm8Instruction {
    mnem: "ldw",
    disassemble,
    operand: Stm8Operands::Alu16(Register16::X, Operand16::Imm(0)),
    handler: |insn, s| {
        let (dst, src) = insn.operand.get_alu16();
        let r = src.get_u16(s);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        dst.set_u16(s, r);
    },
};

const MOV: Stm8Instruction = Stm8Instruction {
    mnem: "mov",
    disassemble,
    operand: Stm8Operands::Mov(Operand8::Imm8(0), Operand8::Abs(0)),
    handler: |insn, s| {
        let (src, dst) = insn.operand.get_ld();
        s.zero = src.get_u8(s).map(|r| r == 0);
        s.sign = src.get_u8(s).map(|r| r.leading_zeros() == 0);
        dst.set_u8(s, src.get_u8(s));
    },
};

const MUL: Stm8Instruction = Stm8Instruction {
    mnem: "mul",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        let dst = insn.operand.get_r16();
        let b = dst.get_u16(s);
        let a = s.a;
        if a.is_none() || b.is_none() {
            dst.set_u16(s, None);
            s.carry = None;
            return;
        }
        let product = (a.unwrap() as u16) * (b.unwrap() as u16);
        dst.set_u16(s, Some(product));
        s.carry = Some(false);
    },
};

const NEG: Stm8Instruction = Stm8Instruction {
    mnem: "neg",
    disassemble,
    operand: Stm8Operands::Rmw(Operand8::A),
    handler: |insn, s| {
        let val = insn.operand.get_rmw().get_u8(s);
        let r = val.map(|a| (a ^ 0xff).wrapping_add(1));
        s.carry = val.map(|a| a == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_rmw().set_u8(s, r);
    },
};

const NEGW: Stm8Instruction = Stm8Instruction {
    mnem: "negw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        let a = insn.operand.get_r16().get_u16(s);
        let r = a.map(|a| (a ^ 0xffff).wrapping_add(1));
        s.carry = a.map(|a| a == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.get_r16().set_u16(s, r);
    },
};

const OR: Stm8Instruction = Stm8Instruction {
    mnem: "or",
    disassemble,
    operand: Stm8Operands::Alu8(Operand8::Imm8(0)),
    handler: |insn, s| {
        let val = insn.operand.get_alu8().get_u8(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const INSTRUCTIONS: [Stm8Instruction; 31] = [
    ADC, ADD, ADDW, AND, BCCM, BCP, BCPL, BRES, BSET, CCF, CLR, CLRW, CP, CPW, CPL, CPLW, DEC,
    DECW, DIV, DIVW, EXG, EXGW, INC, INCW, LD, LDW, MOV, MUL, NEG, NEGW, OR
];

impl std::fmt::Display for Operand8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Operand8::A => write!(f, "a"),
            Operand8::Xl => write!(f, "xl"),
            Operand8::Yl => write!(f, "yl"),
            Operand8::Xh => write!(f, "xh"),
            Operand8::Yh => write!(f, "yh"),
            Operand8::Imm8(v) => write!(f, "#${:x}", v),
            Operand8::Abs(v) => write!(f, "#${:x}", v),
        }
    }
}

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

impl Instruction for Stm8Instruction {
    type State = Stm8;
    fn randomize(&mut self) {
        todo!();
    }
    fn length(&self) -> usize {
        // TODO: an actual implementation for this.
        1usize
    }
    fn operate(&self, s: &mut Stm8) {
        (self.handler)(self, s);
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        use rand::seq::SliceRandom;
        let mut insn = *INSTRUCTIONS.choose(&mut rand::thread_rng()).unwrap();
        insn.mutate();
        insn
    }
}

impl Strop for Stm8Instruction {
    fn random() -> Stm8Instruction {
        Stm8Instruction::new()
    }

    fn mutate(&mut self) {
        self.operand.mutate()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn find_it(opcode: &'static str) -> Stm8Instruction {
        for _ in 0..5000 {
            let insn = Stm8Instruction::random();
            let dasm = format!("{}", insn);
            if dasm.contains(opcode) {
                return insn;
            }
        }
        panic!("Could not find opcode {}", opcode);
    }

    #[test]
    fn instruction_set() {
        for opcode in vec![
            "adc", "add", "addw", "and", "bccm", "bcp", "bcpl", "bres", "bset", "ccf", "clr",
            "clrw", "cp", "cpw", "cpl", "cplw", "dec", "decw", "div", "divw", "exg", "exgw", "inc",
            "incw", "ld", "ldw", "mov", "mul", "neg", "negw", "or", "pop", "popw", "push", "pushw",
            "rcf", "rlc", "rlcw", "rlwa", "rrc", "rrcw", "rrwa", "rvf", "sbc", "scf", "sll",
            "sllw", "sra", "sraw", "srl", "srlw", "sub", "subw", "swap", "swapw", "tnz", "tnzw",
            "xor",
        ] {
            find_it(opcode);
        }
    }
}
