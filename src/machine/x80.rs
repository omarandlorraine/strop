use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{Add, AddWithCarry};
use crate::machine::FlowControl;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Width;
use crate::machine::R;
use crate::State;
use rand::random;
use strop::randomly;

#[derive(Clone, Copy)]
pub struct KR580VM1Instruction {
    randomizer: fn(&mut KR580VM1Instruction),
    disassemble: fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&KR580VM1Instruction, &mut State) -> FlowControl,
}

impl std::fmt::Display for KR580VM1Instruction {
    fn fmt(&self, s: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(s)
    }
}

impl Instruction for KR580VM1Instruction {
    fn randomize(&mut self) {
        (self.randomizer)(self);
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, s: &mut State) -> FlowControl {
        (self.handler)(self, s)
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

fn registers_8080(name: &str) -> Result<Datum, &'static str> {
    match name {
        "a" => Ok(Datum::Register(R::A)),
        "b" => Ok(Datum::Register(R::B)),
        "c" => Ok(Datum::Register(R::C)),
        "d" => Ok(Datum::Register(R::D)),
        "e" => Ok(Datum::Register(R::E)),
        "h" => Ok(Datum::Register(R::H)),
        "l" => Ok(Datum::Register(R::L)),
        "bc" => Ok(Datum::RegisterPair(R::B, R::C)),
        "de" => Ok(Datum::RegisterPair(R::D, R::E)),
        "hl" => Ok(Datum::RegisterPair(R::H, R::L)),
        _ => {
            panic!("No such register as {}", name);
        }
    }
}

fn registers_kr580vm1(r: &str) -> Result<Datum, &'static str> {
    if r == "h1" {
        Ok(Datum::Register(R::H1))
    } else if r == "l1" {
        Ok(Datum::Register(R::L1))
    } else if r == "h1l1" {
        Ok(Datum::RegisterPair(R::H1, R::L1))
    } else {
        registers_8080(r)
    }
}

pub const KR580VM1: Machine = Machine {
    name: "kr580vm1",
    reg_by_name: registers_kr580vm1,
};

#[cfg(test)]
mod tests {}
