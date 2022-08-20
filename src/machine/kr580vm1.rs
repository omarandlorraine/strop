#![allow(missing_docs)]

use crate::machine::Instruction;
use std::collections::HashMap;

// some clippy warnings disabled for this module because KR580VM1 support is not there yet.

#[derive(Default, Debug)]
#[allow(dead_code, unused_variables)]
pub struct RegisterPair {
    low: Option<u8>,
    high: Option<u8>,
}

#[derive(Default)]
#[allow(dead_code, unused_variables)]
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
    fn length(&self) -> usize {
        todo!()
    }
    fn operate(&self, s: &mut KR580VM1) {
        (self.handler)(self, s)
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
