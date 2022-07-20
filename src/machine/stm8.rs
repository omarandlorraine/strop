use crate::machine::Instruction;
use crate::machine::Strop;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::random;
use rand::Rng;
use std::collections::HashMap;
use strop::randomly;

#[derive(Clone, Copy)]
enum Operand8 {
    Imm8(u8),
    Abs(u16),
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

impl Operand8 {
    fn get_u8(self, s: &Stm8) -> Option<u8> {
        use Operand8::*;
        match self {
            Imm8(x) => Some(x),
            Abs(addr) => s.read_mem(Some(addr)),
        }
    }

    fn get_i8(self, s: &Stm8) -> Option<i8> {
        self.get_u8(s).map(|v| i8::from_ne_bytes(v.to_ne_bytes()))
    }
}

#[derive(Default)]
pub struct IndexRegister {
    high: Option<u8>,
    low: Option<u8>,
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
    fn read_mem(&self, addr: Option<u16>) -> Option<u8> {
        if let Some(addr) = addr {
            self.m[&addr]
        } else {
            None
        }
    }
}

struct Opcode {
    handler: fn(&Stm8Instruction, &mut Stm8),
    name: &'static str,
}

#[derive(Clone, Copy)]
struct Alu8Operation {
    opcode: &'static str,
    handler: fn(Option<u8>, &mut Stm8),
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
        let carry = carrytests.map(|t| t.leading_zeros() == 0);
        let zero = r.map(|r| r == 0);
        let sign = r.map(|r| r.leading_zeros() == 0);
        let halfcarry = carrytests.map(|t| t & 0x08 != 0);
        let overflow = overflowtests.map(|t| t != 0 && t != -64);
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
    },
};

const AND: Alu8Operation = Alu8Operation {
    opcode: "and",
    handler: |m, s| {
        let r = s.a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r & 0x80 != 0);
    },
};

impl Distribution<Alu8Operation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Alu8Operation {
        randomly!(
            {ADC} {ADD} {AND}
        )
    }
}

#[derive(Clone, Copy)]
pub enum Stm8Instruction {
    Alu8(Alu8Operation, Operand8),
}

impl Strop for Stm8Instruction {
    fn random() -> Stm8Instruction {
        use Stm8Instruction::*;
        randomly!({ Alu8(random(), Operand8::random()) })
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

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
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
    fn operate(&self, _s: &mut Stm8) {
        todo!()
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
}
