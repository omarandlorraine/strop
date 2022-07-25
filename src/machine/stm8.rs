use crate::machine::Instruction;
use crate::machine::Strop;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::random;
use rand::Rng;
use std::collections::HashMap;
use strop::randomly;

#[derive(Clone, Copy)]
pub enum Operand8 {
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

impl Strop for u8 {
    fn random() -> u8 {
        random()
    }

    fn mutate(&mut self) {
        randomly!(
            /* could try incrementing or decrementing */
            { *self += 1 }
            { *self -= 1 }

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
            { *self += 1 }
            { *self -= 1 }

            /* could try flipping a bit */
            {
                let bit_select = 1_u16.rotate_left(rand::thread_rng().gen_range(0..16));
                *self ^= bit_select;
            }
        );
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

impl Operand8 {
    fn get_u8(self, s: &Stm8) -> Option<u8> {
        use Operand8::*;
        match self {
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
}

#[derive(Clone, Copy)]
pub struct Alu8Operation {
    opcode: &'static str,
    handler: fn(Option<u8>, &mut Stm8),
}

#[derive(Clone, Copy)]
pub struct Alu16Operation {
    opcode: &'static str,
    handler: fn(Option<u16>, Register16, &mut Stm8),
}

const ADC: Alu8Operation = Alu8Operation {
    opcode: "adc",
    handler: |val, s| {
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

const ADD: Alu8Operation = Alu8Operation {
    opcode: "add",
    handler: |val, s| {
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

const ADDW: Alu16Operation = Alu16Operation {
    opcode: "addw",
    handler: |val, register, s| {
        let m = val.map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
        let a = s
            .get_register16(register)
            .map(|v| i16::from_ne_bytes(v.to_ne_bytes()));
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
        s.set_register16(register, r.map(|v| u16::from_ne_bytes(v.to_ne_bytes())));
    },
};

const AND: Alu8Operation = Alu8Operation {
    opcode: "and",
    handler: |m, s| {
        let r = s.a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r & 0x80 != 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

impl Distribution<Alu8Operation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Alu8Operation {
        randomly!(
            {ADC} {ADD} {AND}
        )
    }
}

impl Distribution<Alu16Operation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Alu16Operation {
        randomly!({ ADDW })
    }
}

impl Distribution<Register16> for Standard {
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Register16 {
        randomly!(
            {Register16::X} {Register16::Y}
        )
    }
}

#[derive(Clone, Copy)]
pub enum Stm8Instruction {
    Alu8(Alu8Operation, Operand8),
    Alu16(Alu16Operation, Register16, Operand16),
}

impl Strop for Stm8Instruction {
    fn random() -> Stm8Instruction {
        use Stm8Instruction::*;
        randomly!(
        { Alu8(random(), Operand8::random()) }
        { Alu16(random(), random(), Operand16::random()) }
        )
    }

    fn mutate(&mut self) {
        use Stm8Instruction::*;
        match self {
            Alu8(op, operand) => {
                randomly!(
                    {*self = Alu8(random(), *operand); }
                    {*self = Alu8(*op, Operand8::random()); }
                );
            }
            Alu16(op, register, operand) => {
                randomly!(
                    {*self = Alu16(random(), *register, *operand); }
                    {*self = Alu16(*op, *register, Operand16::random()); }
                    {*self = Alu16(*op, random(), *operand); }
                );
            }
        }
    }
}

impl std::fmt::Display for Register16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Register16::X => write!(f, "x"),
            Register16::Y => write!(f, "y"),
        }
    }
}

impl std::fmt::Display for Operand8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Operand8::*;
        match self {
            Imm8(x) => write!(f, "#${:#04x}", x),
            Abs(addr) => write!(f, "${:#06x}", addr),
        }
    }
}

impl std::fmt::Display for Operand16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Operand16::*;
        match self {
            Imm(x) => write!(f, "#${:#06x}", x),
            Abs(addr) => write!(f, "${:#06x}", addr),
        }
    }
}

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Stm8Instruction::*;
        match self {
            Alu8(op, operand) => {
                write!(f, "\t{} a, {}", op.opcode, operand)
            }
            Alu16(op, register, operand) => {
                write!(f, "\t{} {}, {}", op.opcode, register, operand)
            }
        }
    }
}

impl Instruction for Stm8Instruction {
    type State = Stm8;

    fn randomize(&mut self) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }

    fn operate(&self, s: &mut Stm8) {
        use Stm8Instruction::*;
        match self {
            Alu8(op, operand) => (op.handler)(operand.get_u8(s), s),
            Alu16(op, register, operand) => (op.handler)(operand.get_u16(s), *register, s),
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_it(opcode: &'static str) {
        for _ in 0..5000 {
            let insn = Stm8Instruction::random();
            let dasm = format!("{}", insn);
            if dasm.contains(opcode) {
                return;
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
