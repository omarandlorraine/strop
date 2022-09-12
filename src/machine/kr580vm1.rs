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

impl R8 {
    fn prefix(self) -> Prefix {
        match self {
            R8::H1 => Prefix::Rs,
            R8::L1 => Prefix::Rs,
            R8::M1 => Prefix::Rs,
            _ => Prefix::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prefix {
    None,
    Rs,
    Mb,
    MbRs,
}

impl Prefix {
    fn cycles_tacts(self) -> (u8, u8) {
        match self {
            Prefix::Rs => (1, 4),
            Prefix::Mb => (1, 4),
            Prefix::MbRs => (2, 8),
            Prefix::None => (0, 0),
        }
    }

    fn length(self) -> usize {
        match self {
            Prefix::Rs => 1usize,
            Prefix::Mb => 1usize,
            Prefix::MbRs => 2usize,
            _ => 0usize,
        }
    }

    fn is_mb(self) -> bool {
        match self {
            Prefix::Mb => true,
            Prefix::MbRs => true,
            _ => false,
        }
    }

    fn requires_kr580vm1(self) -> bool {
        match self {
            Prefix::None => false,
            _ => true,
        }
    }

    fn mb() -> Prefix {
        randomly!(
            { Prefix::None }
            { Prefix::Mb }
        )
    }

    fn rs() -> Prefix {
        randomly!(
            { Prefix::None }
            { Prefix::Rs }
        )
    }

    fn mb_rs() -> Prefix {
        randomly!(
            { Prefix::None }
            { Prefix::Mb }
            { Prefix::Rs }
            { Prefix::MbRs }
        )
    }
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

impl From<&RegisterPairBorD> for R16 {
    fn from(item: &RegisterPairBorD) -> Self {
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
    fn prefix(self) -> Prefix {
        match self {
            R16::H1L1 => Prefix::Rs,
            _ => Prefix::None,
        }
    }

    fn split(self) -> (R8, R8) {
        use R16::*;
        use R8::*;
        match self {
            BC => (B, C),
            DE => (D, E),
            HL => (H, L),
            H1L1 => (H1, L1),
            SP => panic!(),
        }
    }

    fn l(self) -> R8 {
        self.split().1
    }

    fn h(self) -> R8 {
        self.split().0
    }

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
    tacts: u64,
    cycles: u64,
}

impl KR580VM1 {
    fn cycles_tacts(&mut self, ct: (u8, u8)) {
        self.cycles += ct.0 as u64;
        self.tacts += ct.1 as u64;
    }

    fn set_mf(&mut self, val: bool) {
        self.mf = val;
    }

    fn read_mem(&mut self, addr: Option<u16>) -> Option<u8> {
        if let Some(a) = addr {
            *self.m.get(&a).unwrap_or(&None)
        } else {
            None
        }
    }

    fn read_mem1(&mut self, addr: Option<u16>) -> Option<u8> {
        if let Some(a) = addr {
            *self.m1.get(&a).unwrap_or(&None)
        } else {
            None
        }
    }

    fn read_memp(&mut self, pfx: Prefix, addr: Option<u16>) -> Option<u8> {
        if pfx.is_mb() {
            self.read_mem1(addr)
        } else {
            self.read_mem(addr)
        }
    }

    fn write_mem(&mut self, addr: Option<u16>, val: Option<u8>) {
        if let Some(a) = addr {
            self.m.insert(a, val);
        }
    }

    fn write_mem1(&mut self, addr: Option<u16>, val: Option<u8>) {
        if let Some(a) = addr {
            self.m1.insert(a, val);
        }
    }

    fn write_memp(&mut self, pfx: Prefix, addr: Option<u16>, val: Option<u8>) {
        if pfx.is_mb() {
            self.write_mem1(addr, val);
        } else {
            self.write_mem(addr, val);
        }
    }

    fn get_addr(&mut self, register_pair: R16) -> Option<u16> {
        match register_pair {
            R16::BC => self.b.get_u16(),
            R16::DE => self.d.get_u16(),
            R16::HL => self.h.get_u16(),
            R16::H1L1 => self.h1.get_u16(),
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
            R8::H1 => self.h1.high = val,
            R8::L1 => self.h1.low = val,
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

    fn get8(&mut self, reg: R8) -> Option<u8> {
        match reg {
            R8::A => self.a,
            R8::B => self.b.high,
            R8::C => self.b.low,
            R8::D => self.d.high,
            R8::E => self.d.low,
            R8::H => self.h.high,
            R8::L => self.h.low,
            R8::H1 => self.h1.high,
            R8::L1 => self.h1.low,
            R8::M => {
                let addr = self.get_addr(R16::HL);
                self.read_mem(addr)
            }
            R8::M1 => {
                let addr = self.get_addr(R16::HL);
                self.read_mem1(addr)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KR580VM1Instruction {
    /// 6.1.1 Данная команда пересылает содержимое устройства-источника в устройство-приемник.
    Mov(Prefix, R8, R8),

    /// 6.1.2 Данная команда пересылает байт непосредственных данных в устройство-приемник.
    Mvi(Prefix, R8, u8),

    /// 6.1.3 Данная команда пересылает два байта непосредственных данных в регистровую память.
    Lxi(Prefix, R16, u8, u8),

    /// 6.1.4 Данная команда пересылает содержимое ячейки памяти в аккумулятор.
    /// Адрес ячейки памяти находится в регистровой паре BC или DE.
    Ldax(Prefix, RegisterPairBorD),

    /// 6.1.5. Данная команда пересылает содержимое аккумулятора в ячейку памяти.
    /// Адрес ячейки памяти находится в регистровой паре BC или DE.
    Stax(Prefix, RegisterPairBorD),
}

impl KR580VM1Instruction {
    /// True if the program ever uses a KR580VM1 extension (i.e. not Intel 8080 compatible)
    fn requires_kr580vm1(self) -> bool {
        use KR580VM1Instruction::*;
        match self {
            Mov(pfx, dst, src) => pfx.requires_kr580vm1(),
            Mvi(pfx, dst, src) => pfx.requires_kr580vm1(),
            Lxi(pfx, dst, h, l) => pfx.requires_kr580vm1(),
            Ldax(pfx, r) => pfx.requires_kr580vm1(),
            Stax(pfx, r) => pfx.requires_kr580vm1(),
        }
    }
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
            Mov(pfx, dst, src) => write!(f, "\t{}mov {}, {}", pfx, dst, src),
            Mvi(pfx, dst, src) => write!(f, "\t{}mvi {}, {}", pfx, dst, src),
            Lxi(pfx, dst, h, l) => write!(f, "\t{}lxi {}, {:02x}{:02x}h", pfx, dst, h, l),
            Ldax(pfx, rp) => write!(f, "\t{}ldax {}", pfx, rp),
            Stax(pfx, rp) => write!(f, "\t{}stax {}", pfx, rp),
        }
    }
}

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Prefix::*;
        match self {
            None => write!(f, ""),
            Rs => write!(f, "rs "),
            Mb => write!(f, "mb "),
            MbRs => write!(f, "rs mb "),
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
            KR580VM1Instruction::Mvi(pfx, dst, src) => {
                randomly!(
                { src.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mov(*pfx, *dst, R8::random()) }
                );
            }
            KR580VM1Instruction::Mov(pfx, dst, src) => {
                randomly!(
                { src.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mvi(*pfx, *dst, random()) }
                );
            }
            KR580VM1Instruction::Lxi(pfx, dst, h, l) => {
                randomly!(
                { l.mutate() }
                { h.mutate() }
                { dst.mutate() }
                { *self = KR580VM1Instruction::Mvi(*pfx, dst.pick_one(), random()) }
                );
            }
            KR580VM1Instruction::Ldax(pfx, rp) => {
                randomly!(
                    { rp.mutate() }
                    {*self = KR580VM1Instruction::Mvi(*pfx, R8::A, random())}
                    {*self = KR580VM1Instruction::Mov(*pfx, R8::A, R8::random())}
                );
            }
            KR580VM1Instruction::Stax(pfx, rp) => rp.mutate(),
        }
    }

    fn length(&self) -> usize {
        match self {
            KR580VM1Instruction::Mov(pfx, dst, src) => pfx.length() + 1,
            KR580VM1Instruction::Mvi(pfx, dst, src) => pfx.length() + 1,
            KR580VM1Instruction::Lxi(pfx, dst, _, _) => pfx.length() + 3,
            KR580VM1Instruction::Ldax(pfx, _) => pfx.length() + 1,
            KR580VM1Instruction::Stax(pfx, _) => pfx.length() + 1,
        }
    }

    fn operate(&self, s: &mut KR580VM1) {
        match self {
            KR580VM1Instruction::Mov(pfx, dst, src) => {
                let r = s.get8(*src);
                s.load8(*dst, r);
                s.cycles_tacts((1, 5));
                s.cycles_tacts(pfx.cycles_tacts());
            }
            KR580VM1Instruction::Mvi(pfx, dst, src) => {
                s.load8(*dst, Some(*src));
            }
            KR580VM1Instruction::Lxi(pfx, dst, h, l) => {
                s.load8(dst.h(), Some(*h));
                s.load8(dst.l(), Some(*l));
            }
            KR580VM1Instruction::Ldax(pfx, r) => {
                let addr = s.get_addr(r.into());
                let r = s.read_memp(*pfx, addr);
                s.load8(R8::A, r);
            }
            KR580VM1Instruction::Stax(pfx, r) => {
                let addr = s.get_addr(r.into());
                let r = s.get8(R8::A);
                s.write_memp(*pfx, addr, r);
            }
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        use KR580VM1Instruction::*;

        randomly!(
        { Mvi(Prefix::mb_rs(), R8::random(), random()) }
        { Mov(Prefix::mb_rs(), R8::random(), R8::random()) }
        { Lxi(Prefix::rs(), R16::random(), random(), random()) }
        { Ldax(Prefix::mb(), RegisterPairBorD::random()) }
        { Stax(Prefix::mb(), RegisterPairBorD::random()) }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::machine::kr580vm1::KR580VM1Instruction;
    use crate::machine::kr580vm1::Prefix;
    use crate::machine::kr580vm1::KR580VM1;
    use crate::machine::kr580vm1::R8;
    use crate::machine::Instruction;

    fn find_it(opcode: &'static str) -> KR580VM1Instruction {
        for _ in 0..5000 {
            let insn = KR580VM1Instruction::new();
            let dasm = format!("{}", insn);
            if dasm.trim().starts_with(opcode) {
                return insn;
            }
        }
        panic!("Could not find opcode {}", opcode);
    }

    fn dont_find_it(opcode: &'static str) {
        // Use this function to assert that an instruction does not exist in the KR580VM1.
        for _ in 0..5000 {
            let insn = KR580VM1Instruction::new();
            let dasm = format!("{}", insn);
            if dasm.trim().starts_with(opcode) {
                panic!("this instruction shouldn't exist {:?}", insn);
            }
        }
    }

    fn test_insn(insn: KR580VM1Instruction, cycles: u64, tacts: u64, req_kr: bool) {
        if req_kr {
            assert!(
                insn.requires_kr580vm1(),
                "The instruction '{}' should report as requiring the KR580VM1 extension",
                insn
            );
        } else {
            assert!(
                !insn.requires_kr580vm1(),
                "The instruction '{}' should not report as requiring the KR580VM1 extension",
                insn
            );
        }

        let mut state = KR580VM1::default();
        insn.operate(&mut state);

        assert_eq!(state.cycles, cycles);
        assert_eq!(state.tacts, tacts);
    }

    #[test]
    fn mov() {
        assert!(find_it("rs mb mov").requires_kr580vm1());
        assert!(find_it("mb mov").requires_kr580vm1());
        assert!(find_it("rs mov").requires_kr580vm1());
        assert!(!find_it("mov").requires_kr580vm1());

        let mov_a_d = KR580VM1Instruction::Mov(Prefix::None, R8::A, R8::D);
        assert_eq!(format!("{}", mov_a_d), "\tmov a, d");
        test_insn(mov_a_d, 1, 5, false);

        let rs_mov_a_d = KR580VM1Instruction::Mov(Prefix::Rs, R8::A, R8::D);
        assert_eq!(format!("{}", rs_mov_a_d), "\trs mov a, d");
        test_insn(rs_mov_a_d, 2, 9, true);
    }

    #[test]
    fn mvi() {
        assert!(find_it("rs mb mvi").requires_kr580vm1());
        assert!(find_it("mb mvi").requires_kr580vm1());
        assert!(find_it("rs mvi").requires_kr580vm1());
        assert!(!find_it("mvi").requires_kr580vm1());
    }

    #[test]
    fn lxi() {
        dont_find_it("rs mb lxi");
        dont_find_it("mb lxi");
        assert!(find_it("rs lxi").requires_kr580vm1());
        assert!(!find_it("lxi").requires_kr580vm1());
    }

    #[test]
    fn ldax() {
        dont_find_it("rs mb ldax");
        assert!(find_it("mb ldax").requires_kr580vm1());
        dont_find_it("rs ldax");
        assert!(!find_it("ldax").requires_kr580vm1());
    }

    #[test]
    fn stax() {
        dont_find_it("rs mb stax");
        assert!(find_it("mb stax").requires_kr580vm1());
        dont_find_it("rs stax");
        assert!(!find_it("stax").requires_kr580vm1());
    }

    #[test]
    fn instruction_set() {
        for opcode in vec![
            "lda", "sta", "lhld", "shld", "lhlx", "shlx", "sphl", "sphl", "xthl", "xchg", "push",
            "pop", "add", "adc", "sub", "sbb", "inr", "inx", "dcr", "dcx", "adi", "aci", "sui",
            "sbi", "dad", "dsub", "daa", "ana", "ani", "anx", "xra", "xri", "xrx", "ora", "ori",
            "orx", "cmp", "cpi", "dcmp", "rlc", "rrc", "rla", "rar", "cma", "cmc",
        ] {
            find_it(opcode);
        }
    }
}
