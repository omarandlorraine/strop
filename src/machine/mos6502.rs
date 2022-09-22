//! The `mos6502` backend, for generating code sequences for the famous 8-bit
//! CPU from 1975. It also supports the later CMOS opcodes and known illegal opcodes
//! present on the NMOS models.

#![warn(missing_debug_implementations, missing_docs)]

use crate::machine::Instruction;
use crate::randomly;
use rand::random;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;

/// The internal state of a 6502
#[derive(Debug, Default)]
pub struct Mos6502 {
    /// The A register
    pub a: Option<u8>,

    /// The X register
    pub x: Option<u8>,

    /// The Y register
    pub y: Option<u8>,

    /// Stack pointer
    pub s: u8,

    /// Memory
    pub heap: HashMap<u16, Option<u8>>,

    /// Carry flag
    pub carry: Option<bool>,

    /// Zero flag
    pub zero: Option<bool>,

    /// Sign flag
    pub sign: Option<bool>,

    /// Overflow flag
    pub overflow: Option<bool>,

    /// Decimal flag
    pub decimal: Option<bool>,

    /// True iff a CMOS-only instruction has been run. (You may want to use this flag in your cost
    /// function to determine if the program will run at all on your device)
    pub requires_cmos: bool,

    /// True iff an illegal instruction has been run. (You may want to use this flag in your cost
    /// function to determine if the program will run reliably on your device)
    pub illegal: bool,

    /// True iff a ROR instruction has been run. (Very early parts do not have this opcode; if you
    /// intend to use such a specimen, then you may want to use this flag in your cost function to
    /// determine if the program will run at all on your device)
    pub requires_ror: bool,
}

impl Mos6502 {
    fn read_mem(&self, addr: u16) -> Option<u8> {
        *self.heap.get(&addr).unwrap_or(&None)
    }

    fn write_mem(&mut self, addr: u16, val: Option<u8>) {
        self.heap.insert(addr, val);
    }

    fn push(&mut self, val: Option<u8>) {
        let addr: u16 = 0x0100 + self.s as u16;
        self.write_mem(addr, val);
        self.s = self.s.wrapping_sub(1);
    }

    fn pull(&mut self) -> Option<u8> {
        self.s = self.s.wrapping_add(1);
        let addr: u16 = 0x0100 + self.s as u16;
        self.read_mem(addr)
    }
}

/// A 6502 instruction's operand
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operand6502 {
    /// Used for implicit instructions, which take no operand
    None,

    /// For RMW instructions operating on the accumulator
    A,

    /// An immediate value
    Immediate(u8),

    /// Absolute addressing mode
    Absolute(u16),
}

impl Operand6502 {
    fn get(self, s: &Mos6502) -> Option<u8> {
        match self {
            Operand6502::None => panic!(),
            Operand6502::A => s.a,
            Operand6502::Immediate(v) => Some(v),
            Operand6502::Absolute(addr) => s.read_mem(addr),
        }
    }
    fn set(self, s: &mut Mos6502, val: Option<u8>) {
        match self {
            Operand6502::None => panic!(),
            Operand6502::A => s.a = val,
            Operand6502::Immediate(_) => panic!(),
            Operand6502::Absolute(addr) => s.write_mem(addr, val),
        }
    }
}

fn no_randomizer(_: &mut Instruction6502) {}

fn immediate_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        Operand6502::Immediate(random())
    }

    insn.operand = match insn.operand {
        Operand6502::None => rnd(),
        Operand6502::A => rnd(),
        Operand6502::Immediate(v) => {
            randomly!(
                {Operand6502::Immediate(v.wrapping_add(1))}
                {Operand6502::Immediate(v.wrapping_sub(1))}
                {let bitsel = 1_u8.rotate_left(rand::thread_rng().gen_range(0..8)); Operand6502::Immediate(v ^ bitsel)}
            )
        }
        Operand6502::Absolute(_) => rnd(),
    }
}

fn aluop_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        randomly!(
            {Operand6502::Immediate(random())}
            {Operand6502::Absolute(random())}
        )
    }

    insn.operand = match insn.operand {
        Operand6502::None => rnd(),
        Operand6502::A => rnd(),
        Operand6502::Immediate(v) => {
            randomly!(
                {Operand6502::Immediate(v.wrapping_add(1))}
                {Operand6502::Immediate(v.wrapping_sub(1))}
                {let bitsel = 1_u8.rotate_left(rand::thread_rng().gen_range(0..8)); Operand6502::Immediate(v ^ bitsel)}
            )
        }
        Operand6502::Absolute(addr) => {
            randomly!(
                {Operand6502::Absolute(addr.wrapping_add(1))}
                {Operand6502::Absolute(addr.wrapping_sub(1))}
            )
        }
    }
}

fn rmwop_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        randomly!(
            {Operand6502::A}
            {Operand6502::Absolute(random())}
        )
    }

    insn.operand = match insn.operand {
        Operand6502::None => rnd(),
        Operand6502::A => rnd(),
        Operand6502::Immediate(_) => rnd(),
        Operand6502::Absolute(addr) => {
            randomly!(
                {Operand6502::Absolute(addr.wrapping_add(1))}
                {Operand6502::Absolute(addr.wrapping_sub(1))}
            )
        }
    }
}

fn absolute_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        randomly!({ Operand6502::Absolute(random()) })
    }

    insn.operand = match insn.operand {
        Operand6502::None => rnd(),
        Operand6502::A => rnd(),
        Operand6502::Immediate(_) => rnd(),
        Operand6502::Absolute(addr) => {
            randomly!(
                {rnd()}
                {Operand6502::Absolute(addr.wrapping_add(1))}
                {Operand6502::Absolute(addr.wrapping_sub(1))}
            )
        }
    }
}

fn store_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        Operand6502::Absolute(random())
    }

    insn.operand = match insn.operand {
        Operand6502::None => rnd(),
        Operand6502::A => rnd(),
        Operand6502::Immediate(_) => rnd(),
        Operand6502::Absolute(addr) => {
            randomly!(
                {Operand6502::Absolute(addr.wrapping_add(1))}
                {Operand6502::Absolute(addr.wrapping_sub(1))}
            )
        }
    }
}

fn disassemble(insn: &Instruction6502, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match insn.operand {
        Operand6502::None => {
            write!(f, "\t{}", insn.mnem)
        }
        Operand6502::A => {
            write!(f, "\t{} a", insn.mnem)
        }
        Operand6502::Immediate(val) => {
            write!(f, "\t{} #${:02x}", insn.mnem, val)
        }
        Operand6502::Absolute(addr) => {
            write!(f, "\t{} ${:04x}", insn.mnem, addr)
        }
    }
}

impl Debug for Instruction6502 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

/// Represents a 6502 Instruction
#[derive(Clone, Copy)]
pub struct Instruction6502 {
    mnem: &'static str,
    randomizer: fn(&mut Instruction6502),
    disassemble: fn(&Instruction6502, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut Mos6502),
    operand: Operand6502,
}

fn compare(insn: &Instruction6502, register: Option<u8>, s: &mut Mos6502) {
    let val = insn.operand.get(s);
    let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let a = register.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let r = a.zip(m).map(|(a, m)| a.wrapping_sub(m));
    s.carry = register.zip(val).map(|(a, m)| a >= m);

    let a_sign = a.map(|a| a.leading_zeros() == 0);
    let b_sign = m.map(|a| a.leading_zeros() == 0);
    let r_sign = r.map(|a| a.leading_zeros() == 0);
    s.overflow = a_sign
        .zip(b_sign)
        .zip(r_sign)
        .map(|((a, b), r)| (a && b && !r) || (!a && !b && r));

    s.zero = r.map(|r| r == 0);
    s.sign = r.map(|r| r.leading_zeros() == 0);
}

fn decrement(val: Option<u8>, s: &mut Mos6502) -> Option<u8> {
    let r = val.map(|v| v.wrapping_sub(1));
    s.zero = r.map(|r| r == 0);
    s.sign = r.map(|r| r.leading_zeros() == 0);
    r
}

fn increment(val: Option<u8>, s: &mut Mos6502) -> Option<u8> {
    let r = val.map(|v| v.wrapping_add(1));
    s.zero = r.map(|r| r == 0);
    s.sign = r.map(|r| r.leading_zeros() == 0);
    r
}

fn add_with_carry(val: Option<u8>, s: &mut Mos6502) -> Option<u8> {
    let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let addition = a
        .zip(m)
        .zip(s.carry)
        .map(|((a, m), c)| a.wrapping_add(m).wrapping_add(if c { 1 } else { 0 }));

    let decimal_adjust = s.decimal.zip(addition).map(|(d, q)| {
        let r = u8::from_ne_bytes(q.to_ne_bytes());
        if d {
            let s1 = if r & 0x0f > 9 { r.wrapping_add(6) } else { r };
            if s1 & 0xf0 > 0x90 {
                s.carry = Some(true);
                s1.wrapping_add(0x60)
            } else {
                s.carry = Some(false);
                s1
            }
        } else {
            r
        }
    });
    let r = decimal_adjust.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
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
    s.overflow = overflowtests.map(|t| t != 0 && t != -64);
    r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()))
}

fn subtract(a: Option<i8>, m: Option<i8>, s: &mut Mos6502) -> Option<u8> {
    let m = m.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let a = a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let subtraction = a
        .zip(m)
        .zip(s.carry)
        .map(|((a, m), c)| a.wrapping_sub(m).wrapping_sub(if c { 0 } else { 1 }));

    let decimal_adjust = s.decimal.zip(subtraction).map(|(d, q)| {
        let r = u8::from_ne_bytes(q.to_ne_bytes());
        if d {
            let s1 = if r & 0x0f > 9 { r.wrapping_sub(6) } else { r };
            if s1 & 0xf0 > 0x90 {
                s.carry = Some(true);
                s1.wrapping_sub(0x60)
            } else {
                s.carry = Some(false);
                s1
            }
        } else {
            r
        }
    });
    let r = decimal_adjust.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
    let carrytests = a
        .zip(m)
        .zip(r)
        .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
    let overflowtests = a
        .zip(m)
        .zip(r)
        .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
    s.carry = carrytests.map(|t| t.leading_zeros() != 0);
    s.zero = r.map(|r| r == 0);
    s.sign = r.map(|r| r.leading_zeros() == 0);
    s.overflow = overflowtests.map(|t| t != 0 && t != -64);
    r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()))
}

const ADC: Instruction6502 = Instruction6502 {
    mnem: "adc",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let result = add_with_carry(val, s);
        s.a = result;
    },
};

const ALR: Instruction6502 = Instruction6502 {
    mnem: "alr",
    randomizer: immediate_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.carry = r.map(|v| v & 0x01 != 0);
        let result = r.map(|v| v.rotate_right(1) & 0x7f);
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
        s.illegal = true;
    },
};

const ANC: Instruction6502 = Instruction6502 {
    mnem: "anc",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.carry = s.sign;
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
        s.illegal = true;
    },
};

const ARR: Instruction6502 = Instruction6502 {
    mnem: "arr",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        let shifted = r.map(|v| v.rotate_right(1) & 0x7f);
        let result = shifted
            .zip(s.carry)
            .map(|(r, c)| r + if c { 0x80 } else { 0 });
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.carry = r.map(|v| v & 0x01 != 0);
        s.a = result;
        s.illegal = true;
    },
};

const AND: Instruction6502 = Instruction6502 {
    mnem: "and",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const ASL: Instruction6502 = Instruction6502 {
    mnem: "asl",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let r = m.map(|a| (a & 0x7f) << 1);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.carry = m.map(|m| m.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const BIT: Instruction6502 = Instruction6502 {
    mnem: "bit",
    randomizer: absolute_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let r = s.a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = m.map(|r| r.leading_zeros() == 0);
        s.overflow = m.map(|r| (r & 0x40) != 0);
    },
};

const CLC: Instruction6502 = Instruction6502 {
    mnem: "clc",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.carry = Some(false);
    },
};

const CLD: Instruction6502 = Instruction6502 {
    mnem: "cld",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.decimal = Some(false);
    },
};

const CLV: Instruction6502 = Instruction6502 {
    mnem: "clv",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.overflow = Some(false);
    },
};

const CMP: Instruction6502 = Instruction6502 {
    mnem: "cmp",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        compare(insn, s.a, s);
    },
};

const CPX: Instruction6502 = Instruction6502 {
    mnem: "cpx",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        compare(insn, s.x, s);
    },
};

const CPY: Instruction6502 = Instruction6502 {
    mnem: "cpy",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        compare(insn, s.y, s);
    },
};

const DCP: Instruction6502 = Instruction6502 {
    mnem: "dcp",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let res = decrement(insn.operand.get(s), s);
        insn.operand.set(s, res);
        compare(insn, s.a, s);
        s.illegal = true;
    },
};

const DEC: Instruction6502 = Instruction6502 {
    mnem: "dec",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let res = decrement(insn.operand.get(s), s);
        insn.operand.set(s, res);
        if insn.operand == Operand6502::A {
            s.requires_cmos = true;
        }
    },
};

const DEX: Instruction6502 = Instruction6502 {
    mnem: "dex",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let res = decrement(s.x, s);
        s.x = res;
    },
};

const DEY: Instruction6502 = Instruction6502 {
    mnem: "dey",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let res = decrement(s.y, s);
        s.y = res;
    },
};

const EOR: Instruction6502 = Instruction6502 {
    mnem: "eor",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a ^ m);

        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const INC: Instruction6502 = Instruction6502 {
    mnem: "inc",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let res = increment(insn.operand.get(s), s);
        insn.operand.set(s, res);
        if insn.operand == Operand6502::A {
            s.requires_cmos = true;
        }
    },
};

const ISC: Instruction6502 = Instruction6502 {
    mnem: "isc",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let res = increment(insn.operand.get(s), s);
        insn.operand.set(s, res);
        let m = res.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = subtract(a, m, s);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
        s.illegal = true;
    },
};

const INX: Instruction6502 = Instruction6502 {
    mnem: "inx",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |_, s| {
        let res = increment(s.x, s);
        s.x = res;
    },
};

const INY: Instruction6502 = Instruction6502 {
    mnem: "iny",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |_, s| {
        let res = increment(s.y, s);
        s.y = res;
    },
};

const LDA: Instruction6502 = Instruction6502 {
    mnem: "lda",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        s.a = insn.operand.get(s);
        s.zero = s.a.map(|r| r == 0);
        s.sign = s.a.map(|r| r.leading_zeros() == 0);
    },
};

const LAS: Instruction6502 = Instruction6502 {
    mnem: "las",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let result = insn.operand.get(s).map(|v| v & s.s);
        s.a = result;
        s.x = result;
        s.s = result.unwrap_or(s.s);
        s.zero = s.a.map(|r| r == 0);
        s.sign = s.a.map(|r| r.leading_zeros() == 0);
        s.illegal = true;
    },
};

const LAX: Instruction6502 = Instruction6502 {
    mnem: "lax",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        s.a = insn.operand.get(s);
        s.x = s.a;
        s.zero = s.a.map(|r| r == 0);
        s.sign = s.a.map(|r| r.leading_zeros() == 0);
        s.illegal = true;
    },
};

const LDX: Instruction6502 = Instruction6502 {
    mnem: "ldx",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        s.x = insn.operand.get(s);
        s.zero = s.x.map(|r| r == 0);
        s.sign = s.x.map(|r| r.leading_zeros() == 0);
    },
};

const LDY: Instruction6502 = Instruction6502 {
    mnem: "ldy",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        s.y = insn.operand.get(s);
        s.zero = s.y.map(|r| r == 0);
        s.sign = s.y.map(|r| r.leading_zeros() == 0);
    },
};

const LSR: Instruction6502 = Instruction6502 {
    mnem: "lsr",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let before = insn.operand.get(s);
        let result = before.map(|v| v.rotate_right(1) & 0x7f);
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.carry = before.map(|v| v & 0x01 != 0);
        insn.operand.set(s, result);
    },
};

const ORA: Instruction6502 = Instruction6502 {
    mnem: "ora",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a | m);

        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const PHA: Instruction6502 = Instruction6502 {
    mnem: "pha",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.push(s.a);
    },
};

const PHX: Instruction6502 = Instruction6502 {
    mnem: "phx",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.push(s.x);
        s.requires_cmos = true;
    },
};

const PHY: Instruction6502 = Instruction6502 {
    mnem: "phy",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.push(s.y);
        s.requires_cmos = true;
    },
};

const PLA: Instruction6502 = Instruction6502 {
    mnem: "pla",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.a = s.pull();
        s.zero = s.a.map(|r| r == 0);
        s.sign = s.a.map(|r| r.leading_zeros() == 0);
    },
};

const PLX: Instruction6502 = Instruction6502 {
    mnem: "plx",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.x = s.pull();
        s.zero = s.x.map(|r| r == 0);
        s.sign = s.x.map(|r| r.leading_zeros() == 0);
        s.requires_cmos = true;
    },
};

const PLY: Instruction6502 = Instruction6502 {
    mnem: "ply",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.y = s.pull();
        s.zero = s.y.map(|r| r == 0);
        s.sign = s.y.map(|r| r.leading_zeros() == 0);
        s.requires_cmos = true;
    },
};

const ROL: Instruction6502 = Instruction6502 {
    mnem: "rol",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let p = m.map(|a| (a & 0x7f) << 1);
        let r = p.zip(s.carry).map(|(r, c)| r + if c { 1 } else { 0 });
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.carry = m.map(|m| m.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const RLA: Instruction6502 = Instruction6502 {
    mnem: "rla",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let p = m.map(|a| (a & 0x7f) << 1);
        let r = p.zip(s.carry).map(|(r, c)| r + if c { 1 } else { 0 });
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.carry = m.map(|m| m.leading_zeros() == 0);
        insn.operand.set(s, r);
        s.a = r.zip(s.a).map(|(a, m)| a & m);
        s.illegal = true;
    },
};

const ROR: Instruction6502 = Instruction6502 {
    mnem: "ror",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let before = insn.operand.get(s);
        let shifted = before.map(|v| v.rotate_right(1) & 0x7f);
        let result = shifted
            .zip(s.carry)
            .map(|(r, c)| r + if c { 0x80 } else { 0 });
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.carry = before.map(|v| v & 0x01 != 0);
        insn.operand.set(s, result);
        s.requires_ror = true;
    },
};

const RRA: Instruction6502 = Instruction6502 {
    mnem: "rra",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let before = insn.operand.get(s);
        let shifted = before.map(|v| v.rotate_right(1) & 0x7f);
        let result = shifted
            .zip(s.carry)
            .map(|(r, c)| r + if c { 0x80 } else { 0 });
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.carry = before.map(|v| v & 0x01 != 0);

        insn.operand.set(s, result);
        let added = add_with_carry(result, s);
        s.a = added;
        s.illegal = true;
    },
};

const SAX: Instruction6502 = Instruction6502 {
    mnem: "sax",
    randomizer: store_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result = s.a.zip(s.x).map(|(a, x)| a & x);
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, result);
        s.illegal = true;
    },
};

const SBX: Instruction6502 = Instruction6502 {
    mnem: "sbx",
    randomizer: immediate_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result =
            s.a.zip(s.x)
                .map(|(a, x)| (a & x).wrapping_sub(insn.operand.get(s).unwrap()));
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.x = result;
        s.illegal = true;
    },
};

const SBC: Instruction6502 = Instruction6502 {
    mnem: "sbc",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = subtract(a, m, s);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const SEC: Instruction6502 = Instruction6502 {
    mnem: "sec",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.carry = Some(true);
    },
};

const SED: Instruction6502 = Instruction6502 {
    mnem: "sed",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        s.decimal = Some(true);
    },
};

const SLO: Instruction6502 = Instruction6502 {
    mnem: "slo",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let result = val.map(|a| (a & 0x7f) << 1);
        s.carry = val.map(|v| v & 0x01 != 0);

        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a | m);

        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);

        insn.operand.set(s, result);
        s.illegal = true;
    },
};

const SRE: Instruction6502 = Instruction6502 {
    mnem: "sre",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let before = insn.operand.get(s);
        let result = before.map(|v| v.rotate_right(1) & 0x7f);
        s.carry = before.map(|v| v & 0x01 != 0);
        insn.operand.set(s, result);

        let r = s.a.zip(result).map(|(a, m)| a ^ m);

        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, r);
        s.illegal = true;
    },
};

const STA: Instruction6502 = Instruction6502 {
    mnem: "sta",
    randomizer: store_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result = s.a;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, result);
    },
};

const STX: Instruction6502 = Instruction6502 {
    mnem: "stx",
    randomizer: store_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result = s.x;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, result);
    },
};

const STY: Instruction6502 = Instruction6502 {
    mnem: "sty",
    randomizer: store_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result = s.y;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, result);
    },
};

const STZ: Instruction6502 = Instruction6502 {
    mnem: "stz",
    randomizer: store_randomizer,
    disassemble,
    operand: Operand6502::A,
    handler: |insn, s| {
        let result = Some(0u8);
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        insn.operand.set(s, result);
        s.requires_cmos = true;
    },
};

const TAX: Instruction6502 = Instruction6502 {
    mnem: "tax",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = s.a;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.x = result;
    },
};

const TAY: Instruction6502 = Instruction6502 {
    mnem: "tay",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = s.a;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.y = result;
    },
};

const TSX: Instruction6502 = Instruction6502 {
    mnem: "tsx",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = Some(s.s);
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.x = result;
    },
};

const TXA: Instruction6502 = Instruction6502 {
    mnem: "txa",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = s.x;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.a = result;
    },
};

const TYA: Instruction6502 = Instruction6502 {
    mnem: "tya",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = s.y;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.a = result;
    },
};

const TXS: Instruction6502 = Instruction6502 {
    mnem: "txs",
    randomizer: no_randomizer,
    disassemble,
    operand: Operand6502::None,
    handler: |_, s| {
        let result = s.x;
        s.zero = result.map(|r| r == 0);
        s.sign = result.map(|r| r.leading_zeros() == 0);
        s.s = result.unwrap_or(0);
    },
};

const TRB: Instruction6502 = Instruction6502 {
    mnem: "trb",
    randomizer: absolute_randomizer,
    disassemble,
    operand: Operand6502::Absolute(0),
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let r = s.a.zip(m).map(|(a, m)| a & m);
        let w = s.a.zip(m).map(|(a, m)| (a ^ 0xff) & m);
        s.zero = r.map(|r| r == 0);
        insn.operand.set(s, w);
        s.requires_cmos = true;
    },
};

const TSB: Instruction6502 = Instruction6502 {
    mnem: "tsb",
    randomizer: absolute_randomizer,
    disassemble,
    operand: Operand6502::Absolute(0),
    handler: |insn, s| {
        let m = insn.operand.get(s);
        let r = s.a.zip(m).map(|(a, m)| a & m);
        let w = s.a.zip(m).map(|(a, m)| a | m);
        s.zero = r.map(|r| r == 0);
        insn.operand.set(s, w);
        s.requires_cmos = true;
    },
};

const INSTRUCTIONS: [Instruction6502; 58] = [
    ADC, ALR, ANC, AND, ASL, ARR, BIT, CLC, CLD, CLV, CMP, CPX, CPY, DCP, DEC, DEX, DEY, EOR, INC,
    INX, INY, ISC, LAS, LAX, LDA, LDX, LDY, LSR, ORA, PHA, PHX, PHY, PLA, PLX, PLY, RLA, ROL, ROR,
    RRA, SAX, SBC, SBX, SEC, SED, SLO, SRE, STA, STX, STY, STZ, TAX, TAY, TRB, TSB, TSX, TXA, TYA,
    TXS,
];

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

impl Instruction for Instruction6502 {
    type State = Mos6502;
    fn randomize(&mut self) {
        (self.randomizer)(self);
    }
    fn length(&self) -> usize {
        match self.operand {
            Operand6502::None => 1usize,
            Operand6502::A => 1usize,
            Operand6502::Immediate(_) => 2usize,
            Operand6502::Absolute(_) => 3usize,
        }
    }
    fn operate(&self, s: &mut Mos6502) {
        (self.handler)(self, s);
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        use rand::seq::SliceRandom;
        let mut insn = *INSTRUCTIONS.choose(&mut rand::thread_rng()).unwrap();
        insn.randomize();
        insn
    }
}
