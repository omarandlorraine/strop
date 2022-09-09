#![allow(missing_debug_implementations, missing_docs)]

use crate::machine::Instruction;
use crate::machine::Strop;
use std::collections::HashMap;

use crate::randomly;
use rand::random;

// some clippy warnings disabled for this module because KR580VM1 support is not there yet.

#[derive(Default, Debug)]
#[allow(dead_code, unused_variables)]
pub struct RegisterPair {
    low: Option<u8>,
    high: Option<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    H1,
    L1,
    M,
    M1,
}

#[derive(Clone, Copy, Debug)]
pub enum R16 {
    BC,
    DE,
    HL,
    H1L1,
    SP,
}

impl R16 {
    fn pick_one(self) -> R8 {
        use R16::*;
        use R8::*;
        match self {
            BC => randomly!({B} {C}),
            DE => randomly!({D} {E}),
            HL => randomly!({H} {L}),
            H1L1 => randomly!({H1} {L1}),
            SP => R8::random(),
        }
    }
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

#[derive(Clone, Copy, Debug)]
pub enum KR580VM1Instruction {
    /// 6.1.1 Данная команда пересылает содержимое устройства-источника в устройство-приемник.
    Mov(R8, R8),

    /// 6.1.2 Данная команда пересылает байт непосредственных данных в устройство-приемник.
    Mvi(R8, u8),

    /// 6.1.3 Данная команда пересылает два байта непосредственных данных в регистровую память.
    Lxi(R16, u8, u8),
}

impl std::fmt::Display for KR580VM1Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl Strop for R8 {
    fn random() -> Self
    where
        Self: Sized,
    {
        use R8::*;
        randomly!({A} {B} {C} {D} {E} {H} {H1} {L} {L1} {M} {M1})
    }

    fn mutate(&mut self) {
        *self = Self::random();
    }
}

impl Strop for R16 {
    fn random() -> Self
    where
        Self: Sized,
    {
        use R16::*;
        randomly!({BC} {DE} {HL} {H1L1} {SP})
    }

    fn mutate(&mut self) {
        *self = Self::random();
    }
}

impl Instruction for KR580VM1Instruction {
    type State = KR580VM1;
    fn randomize(&mut self) {
        match self {
            KR580VM1Instruction::Mvi(dst, src) => {
                randomly!(
                { src.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mov(*dst, R8::random()) }
                );
            }
            KR580VM1Instruction::Mov(dst, src) => {
                randomly!(
                { src.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mvi(*dst, random()) }
                );
            }
            KR580VM1Instruction::Lxi(dst, h, l) => {
                randomly!(
                { l.mutate() }
                { h.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mvi(dst.pick_one(), random()) }
                );
            }
        }
    }

    fn length(&self) -> usize {
        todo!()
    }

    fn operate(&self, s: &mut KR580VM1) {
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
mod tests {}
