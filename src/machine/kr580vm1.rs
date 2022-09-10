#![allow(missing_debug_implementations, missing_docs)]

use crate::machine::Instruction;
use crate::machine::Strop;
use std::collections::HashMap;

use crate::randomly;
use rand::random;

// some clippy warnings disabled for this module because KR580VM1 support is not there yet.

#[derive(Clone, Copy, Default, Debug)]
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

/// This enum is the type whose possible values are BC or DE. It is used by
/// such instructions as LDAX and STAX.
#[derive(Clone, Copy, Debug)]
pub enum RegisterPairBorD {
    BC,
    DE,
}

impl From<RegisterPairBorD> for R16 {
    fn from(item: RegisterPairBorD) -> Self {
        match item {
            RegisterPairBorD::BC => R16::BC,
            RegisterPairBorD::DE => R16::DE,
        }
    }
}

impl RegisterPair {
    fn get_u16(self) -> Option<u16> {
        self.high
            .zip(self.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }
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
    /// MF is the memory bank selection bit
    mf: bool,
    m: HashMap<u16, Option<u8>>,
    m1: HashMap<u16, Option<u8>>,
    /// True if the program ever uses a KR580VM1 extension (i.e. not Intel 8080 compatible)
    kr580vm1_extension: bool,
}

impl KR580VM1 {
    fn set_mf(&mut self, val: bool) {
        self.kr580vm1_extension = true;
        self.mf = val;
    }

    fn write_mem(&mut self, addr: Option<u16>, val: Option<u8>) {
        if let Some(a) = addr {
            self.m.insert(a, val);
        }
    }

    fn write_mem1(&mut self, addr: Option<u16>, val: Option<u8>) {
        self.kr580vm1_extension = true;
        if let Some(a) = addr {
            self.m1.insert(a, val);
        }
    }

    fn get_addr(&mut self, register_pair: R16) -> Option<u16> {
        match register_pair {
            R16::BC => self.b.get_u16(),
            R16::DE => self.d.get_u16(),
            R16::HL => self.h.get_u16(),
            R16::H1L1 => {
                self.kr580vm1_extension = true;
                self.h1.get_u16()
            }
            R16::SP => self.sp,
        }
    }

    fn load8(&mut self, reg: R8, val: Option<u8>) {
        match reg {
            R8::A => self.a = val,
            R8::B => self.b.high = val,
            R8::C => self.b.low = val,
            R8::D => self.d.high = val,
            R8::E => self.d.low = val,
            R8::H => self.h.high = val,
            R8::L => self.h.low = val,
            R8::H1 => {
                self.kr580vm1_extension = true;
                self.h1.high = val
            }
            R8::L1 => {
                self.kr580vm1_extension = true;
                self.h1.low = val
            }
            R8::M => {
                let addr = self.get_addr(R16::HL);
                self.write_mem(addr, val);
            }
            R8::M1 => {
                let addr = self.get_addr(R16::HL);
                self.write_mem1(addr, val);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KR580VM1Instruction {
    /// 6.1.1 Данная команда пересылает содержимое устройства-источника в устройство-приемник.
    Mov(R8, R8),

    /// 6.1.2 Данная команда пересылает байт непосредственных данных в устройство-приемник.
    Mvi(R8, u8),

    /// 6.1.3 Данная команда пересылает два байта непосредственных данных в регистровую память.
    Lxi(R16, u8, u8),

    /// 6.1.4 Данная команда пересылает содержимое ячейки памяти в аккумулятор.
    /// Адрес ячейки памяти находится в регистровой паре BC или DE.
    Ldax(RegisterPairBorD),

    /// 6.1.5. Данная команда пересылает содержимое аккумулятора в ячейку памяти.
    /// Адрес ячейки памяти находится в регистровой паре BC или DE.
    Stax(RegisterPairBorD),
}

impl std::fmt::Display for R8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use R8::*;
        match self {
            A => write!(f, "a"),
            B => write!(f, "b"),
            C => write!(f, "c"),
            D => write!(f, "d"),
            E => write!(f, "e"),
            H => write!(f, "h"),
            L => write!(f, "l"),
            H1 => write!(f, "h1"),
            L1 => write!(f, "l1"),
            M => write!(f, "m"),
            M1 => write!(f, "m1"),
        }
    }
}

impl std::fmt::Display for R16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use R16::*;
        match self {
            BC => write!(f, "b"),
            DE => write!(f, "d"),
            HL => write!(f, "h"),
            H1L1 => write!(f, "h1"),
            SP => write!(f, "sp"),
        }
    }
}

impl std::fmt::Display for RegisterPairBorD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use RegisterPairBorD::*;
        match self {
            BC => write!(f, "b"),
            DE => write!(f, "d"),
        }
    }
}

impl std::fmt::Display for KR580VM1Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use KR580VM1Instruction::*;
        match self {
            Mov(dst, src) => write!(f, "\tmov {}, {}", dst, src),
            Mvi(dst, src) => write!(f, "\tmvi {}, {}", dst, src),
            Lxi(dst, h, l) => write!(f, "\tlxi {}, {:02x}{:02x}h", dst, h, l),
            Ldax(rp) => write!(f, "\tldax {}", rp),
            Stax(rp) => write!(f, "\tstax {}", rp),
        }
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

impl Strop for RegisterPairBorD {
    fn random() -> Self
    where
        Self: Sized,
    {
        use RegisterPairBorD::*;
        randomly!({BC} {DE})
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
            KR580VM1Instruction::Ldax(rp) => {
                randomly!(
                    { rp.mutate() }
                    {*self = KR580VM1Instruction::Mvi(R8::A, random())}
                    {*self = KR580VM1Instruction::Mov(R8::A, R8::random())}
                );
            }
            KR580VM1Instruction::Stax(rp) => rp.mutate(),
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
        use KR580VM1Instruction::*;

        randomly!(
        { Mvi(R8::random(), random()) }
        { Mov(R8::random(), R8::random()) }
        { Lxi(R16::random(), random(), random()) }
        { Ldax(RegisterPairBorD::random()) }
        { Stax(RegisterPairBorD::random()) }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::machine::kr580vm1::KR580VM1Instruction;
    use crate::machine::Instruction;

    fn find_it(opcode: &'static str) -> KR580VM1Instruction {
        for _ in 0..5000 {
            let insn = KR580VM1Instruction::new();
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
            "mov", "mvi", "lxi", "ldax", "stax", "lda", "sta", "lhld", "shld", "lhlx", "shlx",
            "sphl", "sphl", "xthl", "xchg", "push", "pop", "add", "adc", "sub", "sbb", "inr",
            "inx", "dcr", "dcx", "adi", "aci", "sui", "sbi", "dad", "dsub", "daa", "ana", "ani",
            "anx", "xra", "xri", "xrx", "ora", "ori", "orx", "cmp", "cpi", "dcmp", "rlc", "rrc",
            "rla", "rar", "cma", "cmc",
        ] {
            find_it(opcode);
        }
    }
}
