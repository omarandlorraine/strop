use crate::machine::rand::prelude::SliceRandom;
use crate::machine::Instruction;
use rand::random;
use std::collections::HashMap;

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
}

struct Opcode {
    handler: fn(&Stm8Instruction, &mut Stm8),
    name: &'static str,
}

#[derive(Clone, Copy)]
pub struct Stm8Instruction {
    randomizer: fn(&mut Self),
    disassemble: fn(&Self, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    implementation: fn(&Self, &mut Stm8),
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
    fn random() -> Self
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
