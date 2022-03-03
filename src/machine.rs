use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
use std::collections::HashMap;
extern crate rand;
use rand::random;

#[derive(Clone, Copy, PartialEq)]
pub enum Mos6502Variant {
    Nmos,
    Ricoh2a03,
    Cmos,
    IllegalInstructions,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Motorola8BitVariant {
    Motorola6800,
    Motorola6801,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PicVariant {
    Pic12,
    Pic14,
    Pic16,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PreX86Variant {
    ZilogZ80,
    I8080,
    I8085,
    KR580VM1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Machine {
    Mos6502(Mos6502Variant),
    Motorola6800(Motorola8BitVariant),
    Pic(PicVariant),
    PreX86(PreX86Variant),
}

#[derive(Clone, Copy)]
pub struct Instruction {
    pub operation: Operation,
    randomizer: fn(Machine) -> Operation,
    machine: Machine
}

enum Width {
    Width8, Width16
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn transfer(f: &mut std::fmt::Formatter<'_>, from: R, to: R) -> std::fmt::Result {
            fn name(r: R) -> &'static str {
                match r {
                    R::A => { "a" }
                    R::B => { "b" }
                    R::Xl => { "x" }
                    R::Yl => { "y" }
                    _ => { panic!() }
                }
            }
            write!(f, "\tt{}{}", name(from), name(to))
        }

        match (self.machine, self.operation) {
            (Machine::Mos6502(_), Operation::Move(Datum::Register(from), Datum::Register(to))) => { transfer(f, from, to) }
            (Machine::Motorola6800(_), Operation::Move(Datum::Register(from), Datum::Register(to))) => { transfer(f, from, to) }
            _ => { write!(f, "{:?}", self.operation) }
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum R {
    A,
    B, C, D, E, H, L, H1, L1,
    Xh, Xl,
    Yh, Yl,
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Datum {
    Register(R),
    RegisterPair(R, R),
    Imm8(i8),
    Absolute(u16),
    Zero,
}

impl Datum {
    fn width(&self) -> Width {
        match self {
            Self::Register(_) => { Width::Width8 }
            Self::RegisterPair(_, _) => { Width::Width16 }
            Self::Imm8(_) => { Width::Width8 }
            Self::Absolute(_) => { Width::Width8 }
            Self::Zero => { Width::Width8 }
        }
    }
}

impl Machine {
    pub fn register_by_name(self, name: &str) -> Datum {
        match self {
            Machine::Mos6502(_) => match name {
                "a" => Datum::Register(R::A),
                "x" => Datum::Register(R::Xl),
                "y" => Datum::Register(R::Yl),
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::Motorola6800(_) => match name {
                "a" => Datum::Register(R::A),
                "b" => Datum::Register(R::B),
                "ix" => Datum::RegisterPair(R::Xh, R::Xl),
                "iy" => Datum::RegisterPair(R::Yh, R::Yl),
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::Pic(_) => match name {
                "w" => Datum::Register(R::A),
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::PreX86(variant) => {
                if variant == PreX86Variant::KR580VM1 {
                    if name == "h1" { return Datum::Register(R::H1); }
                    if name == "l1" { return Datum::Register(R::L1); }
                    if name == "h1l1" { return Datum::RegisterPair(R::H1, R::L1); }
                }
                match name {
                    "a" => Datum::Register(R::A),
                    "b" => Datum::Register(R::B),
                    "c" => Datum::Register(R::C),
                    "d" => Datum::Register(R::D),
                    "e" => Datum::Register(R::E),
                    "h" => Datum::Register(R::H),
                    "l" => Datum::Register(R::L),
                    "bc" => Datum::RegisterPair(R::B, R::C),
                    "de" => Datum::RegisterPair(R::D, R::E),
                    "hl" => Datum::RegisterPair(R::H, R::L),
                    _ => {
                        panic!("No such register as {}", name);
                    }
                }
            }
        }
    }
}

pub fn bitwise_and(reg: Option<i8>, a: Option<i8>) -> (Option<i8>, Option<bool>) {
    if let Some(operand) = a {
        if let Some(r) = reg {
            return (Some(r & operand), Some(r & operand == 0));
        }
    }
    (None, None)
}

pub fn bitwise_xor(reg: Option<i8>, a: Option<i8>) -> (Option<i8>, Option<bool>) {
    if let Some(operand) = a {
        if let Some(r) = reg {
            return (Some(r ^ operand), Some(r ^ operand == 0));
        }
    }
    (None, None)
}

#[allow(clippy::many_single_char_names)]
pub fn add_to_reg16(
    reg: Option<i16>,
    a: Option<i16>,
    carry: Option<bool>,
) -> (
    Option<i16>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
) {
    // The return values are the result of the addition, then the flags, carry, zero, sign, overflow, half-carry.
    if let Some(operand) = a {
        if let Some(r) = reg {
            if let Some(c) = carry {
                let v = operand.wrapping_add(if c { 1 } else { 0 });
                let result = r.wrapping_add(v);
                let z = result == 0;
                let c = r.checked_add(v).is_none();
                let n = result < 0;
                let o = (r < 0 && v < 0 && result >= 0) || (r > 0 && v > 0 && result <= 0);
                let h = ((r ^ v ^ result) & 0x10) == 0x10;
                (Some(result), Some(c), Some(z), Some(n), Some(o), Some(h))
            } else {
                (None, None, None, None, None, None)
            }
        } else {
            (None, None, None, None, None, None)
        }
    } else {
        (None, None, None, None, None, None)
    }
}

#[allow(clippy::many_single_char_names)]
pub fn add_to_reg8(
    reg: Option<i8>,
    a: Option<i8>,
    carry: Option<bool>,
) -> (
    Option<i8>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
    Option<bool>,
) {
    // The return values are the result of the addition, then the flags, carry, zero, sign, overflow, half-carry.
    if let Some(operand) = a {
        if let Some(r) = reg {
            if let Some(c) = carry {
                let v = operand.wrapping_add(if c { 1 } else { 0 });
                let result = r.wrapping_add(v);
                let z = result == 0;
                let c = r.checked_add(v).is_none();
                let n = result < 0;
                let o = (r < 0 && v < 0 && result >= 0) || (r > 0 && v > 0 && result <= 0);
                let h = ((r ^ v ^ result) & 0x10) == 0x10;
                (Some(result), Some(c), Some(z), Some(n), Some(o), Some(h))
            } else {
                (None, None, None, None, None, None)
            }
        } else {
            (None, None, None, None, None, None)
        }
    } else {
        (None, None, None, None, None, None)
    }
}

fn decimal_adjust(
    accumulator: Option<i8>,
    carry: Option<bool>,
    halfcarry: Option<bool>,
) -> Option<i8> {
    fn nybble(val: i8, flag: Option<bool>) -> Option<i8> {
        if val & 0x0f > 0x09 {
            return Some(0x06);
        }
        flag?;
        if flag.unwrap_or(false) {
            return Some(0x06);
        }
        Some(0)
    }

    if let Some(a) = accumulator {
        if let Some(right) = nybble(a, halfcarry) {
            let ar = a.wrapping_add(right);
            nybble(ar >> 4, carry).map(|left| ar.wrapping_add(left << 4))
        } else {
            None
        }
    } else {
        None
    }
}

fn rotate_left_thru_carry(val: Option<i8>, carry: Option<bool>) -> (Option<i8>, Option<bool>) {
    if let Some(v) = val {
        if let Some(c) = carry {
            let high_bit_set = v & -128 != 0;
            let shifted = (v & 0x7f).rotate_left(1);
            return (
                Some(if c { shifted + 1 } else { shifted }),
                Some(high_bit_set),
            );
        }
    }
    (None, None)
}

fn rotate_right_thru_carry(val: Option<i8>, carry: Option<bool>) -> (Option<i8>, Option<bool>) {
    if let Some(v) = val {
        if let Some(c) = carry {
            let low_bit_set = v & 1 != 0;
            let shifted = (v & 0x7f).rotate_right(1);
            return (
                Some(if c { shifted | -128i8 } else { shifted }),
                Some(low_bit_set),
            )
        }
    }
    (None, None)
}

#[test]
fn add_to_reg8_test() {
    assert_eq!(
        add_to_reg8(Some(3), Some(3), Some(false)),
        (Some(6), Some(false), Some(false), Some(false), Some(false), Some(false))
    );
    assert_eq!(
        add_to_reg8(Some(127), Some(1), Some(false)),
        (Some(-128), Some(true), Some(false), Some(true), Some(true), Some(true))
    );
    assert_eq!(add_to_reg8(None, Some(3), Some(false)), (None, None, None, None, None, None));
}

#[derive(Clone, Debug, Copy)]
pub enum ShiftType {
    LeftArithmetic,
    RightArithmetic,
    LeftRotateThroughCarry,
    RightRotateThroughCarry,
}

#[derive(Clone, Debug, Copy)]
#[allow(non_camel_case_types)]
pub enum Operation {
    DecimalAdjustAccumulator,
    Decrement(Datum),
    Increment(Datum),
    Add(Datum, Datum, bool),
    And(Datum, Datum),
    Move(Datum, Datum),
    Shift(ShiftType, Datum),
    Carry(bool)
}

impl Instruction {
    pub fn new(
        machine: Machine,
        randomizer: fn(Machine) -> Operation,
    ) -> Instruction {
        Instruction {
            machine,
            operation: randomizer(machine),
            randomizer,
        }
    }

    pub fn randomize(&mut self) {
        self.operation = (self.randomizer)(self.machine);
    }

    #[allow(clippy::many_single_char_names)]
    pub fn operate(&self, s: &mut State) -> bool {
        match self.operation {
            Operation::Add(source, destination, carry) => {
                if !carry { s.carry = Some(false) };
                let (result, c, z, n, o, h) =
                    add_to_reg8(s.get_i8(source), s.get_i8(destination), s.carry);

                s.set_i8(destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }
            Operation::And(source, destination) => {
                let (result, z) = bitwise_and(s.get_i8(source), s.get_i8(destination));
                s.set_i8(destination, result);
                s.zero = z;
                true
            }
            Operation::Move(source, destination) => {
                s.set_i8(destination, s.get_i8(source));
                true
            }

            Operation::DecimalAdjustAccumulator => {
                s.accumulator = decimal_adjust(s.accumulator, s.carry, s.halfcarry);
                true
            }

            Operation::Increment(register) => {
                match register.width() {
                    Width::Width8 => {
                        let (result, _c, z, n, _o, _h) = add_to_reg8(s.get_i8(register), Some(1), Some(false));
                        s.set_i8(register, result);
                        s.zero = z;
                        s.sign = n;
                    } 
                    Width::Width16 => {
                        let (result, _c, z, n, _o, _h) = add_to_reg16(s.get_i16(register), Some(1), Some(false));
                        s.set_i16(register, result);
                        s.zero = z;
                        s.sign = n;
                    }
                }
                true
            }

            Operation::Decrement(register) => {
                let (result, _c, z, n, _o, _h) =
                    add_to_reg8(s.get_i8(register), Some(-1), Some(false));
                s.set_i8(register, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::Shift(shtype, datum) => match shtype {
                ShiftType::LeftArithmetic => {
                    let (val, c) = rotate_left_thru_carry(s.get_i8(datum), Some(false));
                    s.set_i8(datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::RightArithmetic => {
                    let (val, c) = rotate_right_thru_carry(s.get_i8(datum), Some(false));
                    s.set_i8(datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::RightRotateThroughCarry => {
                    let (val, c) = rotate_right_thru_carry(s.get_i8(datum), s.carry);
                    s.set_i8(datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::LeftRotateThroughCarry => {
                    let (val, c) = rotate_left_thru_carry(s.get_i8(datum), s.carry);
                    s.set_i8(datum, val);
                    s.carry = c;
                    true
                }
            },

            Operation::Carry(b) => {
                s.carry = Some(b);
                true
            }

        }
    }
}

//#[derive(Copy, Clone)]
pub struct State {
    accumulator: Option<i8>,
    reg_b: Option<i8>,
    reg_c: Option<i8>,
    reg_d: Option<i8>,
    reg_e: Option<i8>,
    reg_h: Option<i8>,
    reg_l: Option<i8>,
    reg_h1: Option<i8>,
    reg_l1: Option<i8>,
    xl: Option<i8>,
    yl: Option<i8>,
    xh: Option<i8>,
    yh: Option<i8>,
    zero: Option<bool>,
    carry: Option<bool>,
    sign: Option<bool>,
    overflow: Option<bool>,
    halfcarry: Option<bool>,
    heap: HashMap<u16, Option<i8>>,
}

impl State {
    pub fn new() -> State {
        State {
            accumulator: None,
            reg_b: None,
            reg_c: None,
            reg_d: None,
            reg_e: None,
            reg_h: None,
            reg_l: None,
            reg_h1: None,
            reg_l1: None,
            xl: None,
            yl: None,
            xh: None,
            yh: None,
            zero: None,
            carry: None,
            sign: None,
            overflow: None,
            halfcarry: None,
            heap: HashMap::new(),
        }
    }

    pub fn get_i8(&self, d: Datum) -> Option<i8> {
        match d {
            Datum::Register(x) => {
                match x {
                    R::A => { self.accumulator }
                    R::B => { self.reg_b }
                    R::C => { self.reg_c }
                    R::D => { self.reg_d }
                    R::E => { self.reg_e }
                    R::H => { self.reg_h }
                    R::L => { self.reg_l }
                    R::H1 => { self.reg_h1 }
                    R::L1 => { self.reg_l1 }
                    R::Xl => { self.xl }
                    R::Yl => { self.yl }
                    R::Xh => { self.xh }
                    R::Yh => { self.yh }
                }
            }
            Datum::RegisterPair(_, x) => {
                self.get_i8(Datum::Register(x))
            }
            Datum::Imm8(d) => {Some(d)}
            Datum::Absolute(addr) => {
                if let Some(x) = self.heap.get(&addr) {
                    *x
                } else {
                    None
                }
            }
            Datum::Zero => {
                Some(0)
            }
        }
    }

    pub fn get_i16(&self, d: Datum) -> Option<i16> {
        match d {
            Datum::Register(_) => { self.get_i8(d).map(|x| x as i16) }
            Datum::RegisterPair(x, y) => {
                if let Some(msb) = self.get_i8(Datum::Register(x)) {
                    if let Some(lsb) = self.get_i8(Datum::Register(y)) {
                        return Some(msb as i16 * 256 + lsb as i16);
                    }
                }
                None
            }
            Datum::Imm8(d) => { Some(d as i16) }
            Datum::Absolute(addr) => {
                if let Some(l) = self.heap.get(&addr) {
                    if let Some(h) = self.heap.get(&(addr + 1)) {
                        if let Some(low) = l {
                            if let Some(high) = h {
                                return Some((*high as i16 * 256) + *low as i16);
                            }
                        }
                    }
                }
                None
            }
            Datum::Zero => {
                Some(0)
            }
        }
    }

    pub fn set_i8(&mut self, d: Datum, val: Option<i8>) {
        match d {
            Datum::Register(register) => {
                match register {
                    R::A => { self.accumulator = val; }
                    R::B => { self.reg_b = val; }
                    R::C => { self.reg_c = val; }
                    R::D => { self.reg_d = val; }
                    R::E => { self.reg_e = val; }
                    R::H => { self.reg_h = val; }
                    R::L => { self.reg_l = val; }
                    R::H1 => { self.reg_h1 = val; }
                    R::L1 => { self.reg_l1 = val; }
                    R::Xl => { self.xl = val; }
                    R::Yl => { self.yl = val; }
                    R::Xh => { self.xh = val; }
                    R::Yh => { self.yh = val; }
                }
            }
            Datum::RegisterPair(h, l) => {
                self.set_i8(Datum::Register(l), val);
                self.set_i8(Datum::Register(h), Some(0));
            }
            Datum::Imm8(_) => {panic!()}
            Datum::Absolute(address) => {
                self.heap.insert(address, val);
            }
            Datum::Zero => {}
        }
    }
    pub fn set_i16(&mut self, d: Datum, val: Option<i16>) {
        let high = val.map(|v| (v / 256) as i8);
        let low = val.map(|v| (v % 256) as i8);
        match d {
            Datum::Register(_) => {
                self.set_i8(d, low);
            }
            Datum::RegisterPair(h, l) => {
                self.set_i8(Datum::Register(h), high);
                self.set_i8(Datum::Register(l), low);
            }
            Datum::Imm8(_x) => {
                panic!();
            }
            Datum::Absolute(addr) => {
                self.set_i8(Datum::Absolute(addr + 1), high);
                self.set_i8(Datum::Absolute(addr), low);
            }
            Datum::Zero => {}
        }
    }
}

fn random_immediate() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Imm8(*vs.choose(&mut rand::thread_rng()).unwrap())
}

fn random_absolute() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Absolute(*vs.choose(&mut rand::thread_rng()).unwrap())
}

fn random_r_prex86(_mach: Machine) -> Datum {
    match rand::thread_rng().gen_range(0, 8) {
        0 => {Datum::Register(R::A)}
        1 => {Datum::Register(R::B)}
        2 => {Datum::Register(R::C)}
        3 => {Datum::Register(R::D)}
        4 => {Datum::Register(R::E)}
        5 => {Datum::Register(R::A)} // TODO: this should be (HL) in the zilog syntax; the byte pointed to by HL.
        6 => {Datum::Register(R::H)}
        _ => {Datum::Register(R::L)}
    }
}

fn random_rp_prex86(_mach: Machine) -> Datum {
    match rand::thread_rng().gen_range(0, 3) {
        0 => {Datum::RegisterPair(R::B, R::C)}
        1 => {Datum::RegisterPair(R::D, R::E)}
        _ => {Datum::RegisterPair(R::H, R::L)}
    }
}

fn inc_dec_prex86(mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 4) {
        0 => { Operation::Increment(random_r_prex86(mach)) }
        1 => { Operation::Increment(random_rp_prex86(mach)) }
        2 => { Operation::Decrement(random_r_prex86(mach)) }
        _ => { Operation::Decrement(random_rp_prex86(mach)) }
    }
}

pub fn instr_prex86(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 1) {
        0 => { Instruction::new(mach, inc_dec_prex86) }
        _ => { Instruction::new(mach, |_| Operation::DecimalAdjustAccumulator) }
    }
}

fn random_accumulator_6800() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        Datum::Register(R::B)
    }
}

fn random_source_6800() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn rmw_datum_6800() -> Datum {
    // Data that can be the operand for 6800 operations like ASL and COM
    if random() {
        random_accumulator_6800()
    } else {
        random_absolute()
    }
}

fn add_6800(_mach: Machine) -> Operation {
    let dst = random_accumulator_6800();
    if dst == Datum::Register(R::A) && random() {
        Operation::Add(Datum::Register(R::B), dst, false) // ABA
    } else {
        Operation::Add(random_source_6800(), dst, random()) // ADCA, ADCB, ADDA, ADDB
    }
}

fn transfers_6800(_mach: Machine) -> Operation {
    if random() {
        Operation::Move(Datum::Register(R::A), Datum::Register(R::B))
    } else {
        Operation::Move(Datum::Register(R::B), Datum::Register(R::A))
    }
}

fn rotates_6800(_mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 4) {
        0 => {Operation::Shift(ShiftType::LeftArithmetic, rmw_datum_6800())}
        1 => {Operation::Shift(ShiftType::RightArithmetic,rmw_datum_6800())}
        2 => {Operation::Shift(ShiftType::LeftRotateThroughCarry, rmw_datum_6800())}
        _ => {Operation::Shift(ShiftType::RightRotateThroughCarry, rmw_datum_6800())}
    }
}

pub fn instr_6800(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 4) {
        0 => { Instruction::new(mach, add_6800) }
        1 => { Instruction::new(mach, transfers_6800) }
        2 => { Instruction::new(mach, |_| Operation::DecimalAdjustAccumulator) }
        _ => { Instruction::new(mach, rotates_6800) }
    }
    // TODO: Add clc, sec, daa, and many other instructions
}

pub fn new_instruction(mach: Machine) -> Instruction {
    match mach {
        Machine::Motorola6800(_) => { instr_6800(mach) }
        Machine::Mos6502(_) => { instr_6502(mach) }
        Machine::PreX86(_) => { instr_prex86(mach) }
        Machine::Pic(_) => { instr_pic(mach) }
    }
}

fn random_source_6502() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn incdec_6502(mach: Machine) -> Operation {
    // the CMOS varieties have inc and dec for accumulator
    // but earlier 6502s can increment and decrement X and Y only.
    let reg = 
    match rand::thread_rng().gen_range(0, if mach == Machine::Mos6502(Mos6502Variant::Cmos) { 3 } else { 2 }) {
        0 => {Datum::Register(R::Xl)}
        1 => {Datum::Register(R::Yl)}
        _ => {Datum::Register(R::A)}
    };
    if random() {
        Operation::Increment(reg)
    } else {
        Operation::Decrement(reg)
    }
}

fn add_6502(_mach: Machine) -> Operation {
    Operation::Add(random_source_6502(), Datum::Register(R::A), true)
}

fn transfers_6502(_mach: Machine) -> Operation {
    let reg = if random() {
        Datum::Register(R::Xl)
    } else {
        Datum::Register(R::Yl)
    };
    if random() {
        Operation::Move(Datum::Register(R::A), reg)
    } else {
        Operation::Move(reg, Datum::Register(R::A))
    }
}

fn loadstore_6502(mach: Machine) -> Operation {
    // TODO: STZ operation for CMOS varieties
    let addr = random_absolute();
    let reg = match rand::thread_rng().gen_range(0, if mach == Machine::Mos6502(Mos6502Variant::Cmos) { 4 } else { 3 }) {
        0 => Datum::Register(R::A),
        1 => Datum::Register(R::Xl),
        2 => Datum::Register(R::Yl),
        _ => Datum::Zero,
    };
    if random() && reg != Datum::Zero {
        Operation::Move(addr, reg)
    } else {
        Operation::Move(reg, addr)
    }
}

fn secl_6502(_mach: Machine) -> Operation {
    Operation::Carry(random())
}
fn shifts_6502(_mach: Machine) -> Operation {
    let sht = match rand::thread_rng().gen_range(0, 4) {
        0 => ShiftType::LeftArithmetic,
        1 => ShiftType::RightArithmetic,
        2 => ShiftType::LeftRotateThroughCarry,
        _ => ShiftType::RightRotateThroughCarry,
    };
    let dat = if random() {
        Datum::Register(R::A)
    }else {
        random_absolute()
    };
    Operation::Shift(sht, dat)
}

fn instr_6502(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 5) {
        0 => { Instruction::new(mach, incdec_6502) }
        1 => { Instruction::new(mach, add_6502) }
        2 => { Instruction::new(mach, transfers_6502) }
        3 => { Instruction::new(mach, shifts_6502) }
        4 => { Instruction::new(mach, loadstore_6502) }
        _ => { Instruction::new(mach, secl_6502) }
    }
    // TODO: Add clc, sec, and many other instructions
}

fn random_accumulator_or_absolute() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        random_absolute()
    }
}

fn inc_dec_pic(_mach: Machine) -> Operation {
    // TODO: These instructions can optionally write to W instead of the F.
    if random() {
        Operation::Increment(random_absolute()) // incf f
    } else {
        Operation::Decrement(random_absolute()) // decf f
    }
}

fn add_pic(mach: Machine) -> Operation {
    let dst = random_accumulator_or_absolute();
    if dst == Datum::Register(R::A) && mach != Machine::Pic(PicVariant::Pic12) && random() {
        // This is an immediate add. Not available on PIC12.
        Operation::Add(random_immediate(), Datum::Register(R::A), false) // addlw k
    } else if random() {
        Operation::Add(random_absolute(), Datum::Register(R::A), false) // addwf f
    } else {
        Operation::Add(Datum::Register(R::A), random_absolute(), false) // addwf f,d
    }
}

fn shifts_pic(_mach: Machine) -> Operation {
    // TODO: These instructions can optionally write to W instead of the F.
    let shtype = if random() {
        ShiftType::RightRotateThroughCarry
    } else {
        ShiftType::LeftRotateThroughCarry
    };
    Operation::Shift(shtype, random_absolute()) // rlf f,d and rrf f,d
}

fn and_pic(_mach: Machine) -> Operation {
    let dst = random_accumulator_or_absolute();
    if dst == Datum::Register(R::A) && random() {
        // andlw
        Operation::And(random_immediate(), dst)
    } else if random() {
        Operation::And(random_absolute(), dst)
    } else {
        Operation::And(dst, random_absolute())
    }
}

fn store_pic(_mach: Machine) -> Operation {
    // TODO: There also is movf f,d, which just updates the Z flag
    match rand::thread_rng().gen_range(0, 4) {
        0 => { Operation::Move(Datum::Zero, random_accumulator_or_absolute()) }              // clrw and clrf f
        1 => { Operation::Move(random_accumulator_or_absolute(), Datum::Register(R::A)) }    // movf f
        2 => { Operation::Move(random_immediate(), Datum::Register(R::A)) }                  // movlw k
        _ => { Operation::Move(Datum::Register(R::A), random_accumulator_or_absolute()) }    // movwf f
    }
}

fn instr_pic(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 5) {
        0 => { Instruction::new(mach, shifts_pic) }
        1 => { Instruction::new(mach, and_pic) }
        2 => { Instruction::new(mach, add_pic) }
        3 => { Instruction::new(mach, store_pic) }
        _ => { Instruction::new(mach, inc_dec_pic) }
    }
    // TODO: Add the following other instructions:
    // bcf bsf btfsc btfss (call) (clrwdt) comf decfsz (goto) incfsz iorlw iorwf (nop) (option) (retlw) (sleep) subwf swapf (tris) xorlw xorwf
}
