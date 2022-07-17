use crate::machine::rand::prelude::SliceRandom;
use crate::machine::Instruction;
use rand::random;
use std::collections::HashMap;
use strop::randomly;

#[derive(Default)]
pub struct RegisterPair {
    low: Option<u8>,
    high: Option<u8>,
}

#[derive(Default)]
pub struct KR580VM1 {
    a: Option<u8>,
    b: RegisterPair,
    d: RegisterPair,
    h: RegisterPair,
    h1: RegisterPair,
    sp: Option<u16>,
    m: HashMap<u16, Option<u8>>,
    m1: HashMap<u16, Option<u8>>,
    /// True if the program ever uses a KR580VM1 extension (i.e. not Intel 8080 compatible)
    kr580vm1_extension: bool,
}

#[derive(Clone, Copy)]
pub struct KR580VM1Instruction {
    randomizer: fn(&mut KR580VM1Instruction),
    disassemble: fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&KR580VM1Instruction, &mut KR580VM1),
}

impl std::fmt::Display for KR580VM1Instruction {
    fn fmt(&self, s: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(s)
    }
}

impl Instruction for KR580VM1Instruction {
    type State = KR580VM1;
    fn randomize(&mut self) {
        (self.randomizer)(self);
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, s: &mut KR580VM1) {
        (self.handler)(self, s)
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
