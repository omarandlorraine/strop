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

#[derive(Clone, Copy, Debug)]
pub enum Operand8 {
    A,
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
            Imm8(_) => panic!(),
            Abs(addr) => s.write_mem(addr, val),
        }
    }

    fn get_u8(self, s: &Stm8) -> Option<u8> {
        use Operand8::*;
        match self {
            A => s.a,
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
    Rmw(Operand8),
    Alu8(Operand8),
    Alu16(Register16, Operand16),
    Bits(u16, usize),
    R16(Register16),
}

impl Stm8Operands {
    fn get_rmw(self) -> Operand8 {
        match self {
            Stm8Operands::Rmw(operand) => operand,
            _ => panic!(),
        }
    }

    fn get_alu8(self) -> Operand8 {
        match self {
            Stm8Operands::Alu8(operand) => operand,
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
        match self {
            Stm8Operands::None => (),
            Stm8Operands::R16(x) => x.mutate(),
            Stm8Operands::Rmw(v) => v.mutate(),
            Stm8Operands::Alu8(v) => v.mutate(),
            Stm8Operands::Alu16(r, v) => randomly!({r.mutate()} {v.mutate()}),
            Stm8Operands::Bits(addr, bit) => randomly!(
                {addr.mutate()}
                {*bit = (*bit + 1) % 8}
                {*bit = (*bit + 7) % 8}
            ),
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
        Stm8Operands::Alu16(r, v) => {
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
    },
};

const CLRW: Stm8Instruction = Stm8Instruction {
    mnem: "clrw",
    disassemble,
    operand: Stm8Operands::R16(Register16::X),
    handler: |insn, s| {
        insn.operand.get_r16().set_u16(s, Some(0));
    },
};

const INSTRUCTIONS: [Stm8Instruction; 12] = [
    ADC, ADD, ADDW, AND, BCCM, BCP, BCPL, BRES, BSET, CCF, CLR, CLRW,
];

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
