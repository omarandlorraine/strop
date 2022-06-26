use crate::machine::rand::prelude::SliceRandom;
use std::collections::HashMap;
extern crate num;
extern crate rand;
use crate::machine::rand::prelude::IteratorRandom;
use num::traits::{WrappingAdd, WrappingSub};
use std::convert::TryInto;

mod stm8;

use crate::machine::stm8::STM8;

use rand::random;

#[derive(Clone, Copy)]
pub struct Machine {
    pub name: &'static str,
    random_insn: fn() -> Instruction,
    reg_by_name: fn(&str) -> Result<Datum, &'static str>,
}

pub const MACHINES: [Machine; 1] = [STM8];

#[derive(Clone, Copy)]
pub struct Instruction {
    mnemonic: &'static str,
    randomizer: fn(&mut Instruction),
    disassemble: fn(&mut std::fmt::Formatter<'_>, &Instruction) -> std::fmt::Result,
    length: usize,
    implementation: fn(&Instruction, &mut State),
    a: Datum,
    b: Datum,
    c: Datum,
}

fn reg_by_name(name: &str) -> Result<Datum, &'static str> {
    if name[0..1] == *"m" {
        let arg = name[1..].to_string();
        if let Ok(addr) = arg.parse::<u16>() {
            return Ok(Datum::Absolute(addr));
        } else {
            return Err("parse error");
        }
    }
    Err("no such register")
}

#[derive(Clone, Copy, Debug)]
pub enum Width {
    Width8,
    Width16,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.disassemble)(f, self)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum R {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    H1,
    L1,
    Xh,
    Xl,
    Yh,
    Yl,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum Datum {
    Nothing,
    Register(R),
    RegisterPair(R, R),
    Imm8(i8),
    Imm16(i16),
    Absolute(u16),
    Zero,
}

impl Datum {
    pub fn width(&self) -> Width {
        match self {
            Self::Register(_) => Width::Width8,
            Self::RegisterPair(_, _) => Width::Width16,
            Self::Imm8(_) => Width::Width8,
            Self::Imm16(_) => Width::Width16,
            Self::Absolute(_) => Width::Width8,
            Self::Zero => Width::Width8,
        }
    }
}

trait Swap {
    fn complement(self) -> Self;
    fn swap(self) -> Self;
    fn shift_left(self, bit_in: Option<bool>) -> (bool, Self);
    fn shift_right(self, bit_in: Option<bool>) -> (bool, Self);
}

trait Bits {
    fn msb() -> Self;
}

impl Bits for u8 {
    fn msb() -> Self {
        2_u8.pow(7)
    }
}

impl Bits for u16 {
    fn msb() -> Self {
        2_u16.pow(15)
    }
}

impl Machine {
    pub fn new_instruction(self) -> Instruction {
        (self.random_insn)()
    }
    pub fn register_by_name(self, name: &str) -> Result<Datum, &'static str> {
        (self.reg_by_name)(name)
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
            );
        }
    }
    (None, None)
}

#[derive(Clone, Debug, Copy)]
pub enum ShiftType {
    LeftArithmetic,
    RightArithmetic,
    LeftRotateThroughCarry,
    RightRotateThroughCarry,
}

#[derive(Clone, Debug, Copy)]
pub enum Test {
    True,
    Minus(bool),
    Zero(bool),
    HalfCarry(bool),
    Overflow(bool),
    Carry(bool),
    UnsignedGreaterThan,
    SignedGreaterThanOrEqual,
    SignedGreaterThan,
    UnsignedLowerThanOrEqual,
    SignedLowerThanOrEqual,
    SignedLowerThan,
    Bit(u16, u8, bool),
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum DyadicOperation {
    Add,
    AddWithCarry,
    And,
    ExclusiveOr,
    Or,
    Subtract,
    SubtractWithBorrow,
    SubtractWithCarry,
    Divide,
    Multiply,
    // TODO: Move the shifts here.
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum MonadicOperation {
    Complement,
    Decrement,
    Increment,
    Negate,
    Swap,
    LeftShiftArithmetic,
    RightShiftArithmetic,
    RightShiftLogical,
    RotateLeftThruCarry,
    RotateRightThruCarry,
    RotateLeftThruAccumulator,
    RotateRightThruAccumulator,
    // TODO: Move the shifts here.
}

#[derive(Clone, Debug, Copy)]
pub enum Operation {
    Monadic(Width, MonadicOperation, Datum, Datum),
    Exchange(Width, Datum, Datum),
    Dyadic(Width, DyadicOperation, Datum, Datum, Datum),
    DecimalAdjustAccumulator,
    BitCompare(Datum, Datum),
    Move(Datum, Datum),
    Shift(ShiftType, Datum),
    Carry(bool),
    Overflow(bool),
    Decimal(bool),
    ComplementCarry,
    BitSet(Datum, u8),
    BitClear(Datum, u8),
    BitComplement(Datum, u8),
    BitCopyCarry(Datum, u8),
    Nop,
}

impl Test {
    fn evaluate(&self, s: &State) -> Option<bool> {
        match self {
            Test::True => Some(true),
            Test::Minus(b) => s.sign.map(|flag| &flag == b),
            Test::Zero(b) => s.zero.map(|flag| &flag == b),
            Test::HalfCarry(b) => s.halfcarry.map(|flag| &flag == b),
            Test::Overflow(b) => s.overflow.map(|flag| &flag == b),
            Test::Carry(b) => s.carry.map(|flag| &flag == b),
            Test::SignedLowerThan => Test::SignedGreaterThanOrEqual.evaluate(s).map(|r| !r),
            Test::UnsignedLowerThanOrEqual => Test::UnsignedGreaterThan.evaluate(s).map(|r| !r),
            Test::SignedLowerThanOrEqual => Test::SignedGreaterThan.evaluate(s).map(|r| !r),
            Test::UnsignedGreaterThan => s
                .overflow
                .map(|v| s.carry.map(|c| !(c || v)))
                .unwrap_or(None),
            Test::SignedGreaterThan => s.overflow.map(|v| s.carry.map(|c| c == v)).unwrap_or(None),
            Test::SignedGreaterThanOrEqual => {
                if let Some(z) = s.zero {
                    if z {
                        Some(true)
                    } else {
                        Test::SignedGreaterThan.evaluate(s)
                    }
                } else {
                    None
                }
            }
            Test::Bit(addr, bit_no, b) => {
                if let Some(byte) = s.get_i8(Datum::Absolute(*addr)) {
                    let val = byte & !(1 << bit_no) != 0;
                    Some(&val == b)
                } else {
                    None
                }
            }
        }
    }
}

impl Instruction {
    pub fn randomize(&mut self) {
        (self.randomizer)(&mut self);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn operate(&self, s: &mut State) {
        (self.implementation)(self, s)
    }
}

/*
pub fn standard_implementation(insn: &Instruction, s: &mut State) -> FlowControl {

    match insn.operation {
        Operation::Exchange(Width::Width8, a, b) => {
            let tmp = s.get_i8(a);
            s.set_i8(a, s.get_i8(b));
            s.set_i8(b, tmp);
            FlowControl::FallThrough
        }
        Operation::Exchange(Width::Width16, a, b) => {
            let tmp = s.get_i16(a);
            s.set_i16(a, s.get_i16(b));
            s.set_i16(b, tmp);
            FlowControl::FallThrough
        }
        Operation::Monadic(Width::Width8, operation, src, dst) => {
            let r = operation.evaluate(s, s.get_i8(src));
            s.set_i8(dst, r);
            FlowControl::FallThrough
        }
        Operation::Monadic(Width::Width16, operation, src, dst) => {
            let r = operation.evaluate(s, s.get_i16(src));
            s.set_i16(dst, r);
            FlowControl::FallThrough
        }
        Operation::Dyadic(Width::Width8, operation, a, b, dst) => {
            let a = s.get_u8(a);
            let b = s.get_u8(b);
            let r = operation.evaluate(s, a, b);
            s.set_u8(dst, r);
            FlowControl::FallThrough
        }
        Operation::Dyadic(Width::Width16, operation, a, b, dst) => {
            let a = s.get_u16(a);
            let b = s.get_u16(b);
            let r = operation.evaluate(s, a, b);
            s.set_u16(dst, r);
            FlowControl::FallThrough
        }
        Operation::Move(source, destination) => {
            s.set_i8(destination, s.get_i8(source));
            FlowControl::FallThrough
        }

        Operation::DecimalAdjustAccumulator => {
            s.accumulator = decimal_adjust(s.accumulator, s.carry, s.halfcarry);
            FlowControl::FallThrough
        }
        Operation::BitCompare(source, destination) => {
            let (result, z) = bitwise_and(s.get_i8(source), s.get_i8(destination));
            if let Some(result) = result {
                s.sign = Some(result < 0);
            } else {
                s.sign = None
            }
            s.zero = z;
            FlowControl::FallThrough
        }
        Operation::Shift(shtype, datum) => match shtype {
            ShiftType::LeftArithmetic => {
                let (val, c) = rotate_left_thru_carry(s.get_i8(datum), Some(false));
                s.set_i8(datum, val);
                s.carry = c;
                FlowControl::FallThrough
            }
            ShiftType::RightArithmetic => {
                let (val, c) = rotate_right_thru_carry(s.get_i8(datum), Some(false));
                s.set_i8(datum, val);
                s.carry = c;
                FlowControl::FallThrough
            }
            ShiftType::RightRotateThroughCarry => {
                let (val, c) = rotate_right_thru_carry(s.get_i8(datum), s.carry);
                s.set_i8(datum, val);
                s.carry = c;
                FlowControl::FallThrough
            }
            ShiftType::LeftRotateThroughCarry => {
                let (val, c) = rotate_left_thru_carry(s.get_i8(datum), s.carry);
                s.set_i8(datum, val);
                s.carry = c;
                FlowControl::FallThrough
            }
        },
        Operation::Overflow(b) => {
            s.overflow = Some(b);
            FlowControl::FallThrough
        }
        Operation::Decimal(b) => {
            s.decimal = Some(b);
            FlowControl::FallThrough
        }
        Operation::Carry(b) => {
            s.carry = Some(b);
            FlowControl::FallThrough
        }
        Operation::ComplementCarry => {
            if let Some(b) = s.carry {
                s.carry = Some(!b)
            }
            FlowControl::FallThrough
        }
        Operation::BitSet(d, b) => {
            if let Some(v) = s.get_i8(d) {
                let new = v | (1 << b);
                s.set_i8(d, Some(new));
            }
            FlowControl::FallThrough
        }
        Operation::BitClear(d, b) => {
            if let Some(v) = s.get_i8(d) {
                let new = v & !(1 << b);
                s.set_i8(d, Some(new));
            }
            FlowControl::FallThrough
        }
        Operation::BitComplement(d, b) => {
            if let Some(v) = s.get_i8(d) {
                let new = v ^ (1 << b);
                s.set_i8(d, Some(new));
            }
            FlowControl::FallThrough
        }
        Operation::BitCopyCarry(d, b) => {
            if let (Some(v), Some(c)) = (s.get_i8(d), s.carry) {
                let new = if c { v & !(1 << b) } else { v | (1 << b) };
                s.set_i8(d, Some(new));
            } else {
                s.set_i8(d, None);
            }
            FlowControl::FallThrough
        }
        Operation::Jump(test, flowcontrol) => {
            if let Some(b) = test.evaluate(s) {
                if b {
                    flowcontrol
                } else {
                    FlowControl::FallThrough
                }
            } else {
                FlowControl::Invalid
            }
        }
        Operation::Nop => FlowControl::FallThrough,
    }
}

*/

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
    halfcarry: Option<bool>,
    overflow: Option<bool>,
    decimal: Option<bool>,
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
            decimal: None,
            heap: HashMap::new(),
        }
    }

    pub fn get_i8(&self, d: Datum) -> Option<i8> {
        match d {
            Datum::Register(x) => match x {
                R::A => self.accumulator,
                R::B => self.reg_b,
                R::C => self.reg_c,
                R::D => self.reg_d,
                R::E => self.reg_e,
                R::H => self.reg_h,
                R::L => self.reg_l,
                R::H1 => self.reg_h1,
                R::L1 => self.reg_l1,
                R::Xl => self.xl,
                R::Yl => self.yl,
                R::Xh => self.xh,
                R::Yh => self.yh,
            },
            Datum::RegisterPair(_, x) => self.get_i8(Datum::Register(x)),
            Datum::Imm16(d) => d.try_into().ok(),
            Datum::Imm8(d) => Some(d),
            Datum::Absolute(addr) => {
                if let Some(x) = self.heap.get(&addr) {
                    *x
                } else {
                    None
                }
            }
            Datum::Zero => Some(0),
        }
    }

    pub fn get_u8(&self, d: Datum) -> Option<u8> {
        self.get_i8(d).map(|v| u8::from_ne_bytes(v.to_ne_bytes()))
    }

    pub fn set_u8(&mut self, d: Datum, val: Option<u8>) {
        self.set_i8(d, val.map(|v| i8::from_ne_bytes(v.to_ne_bytes())));
    }

    pub fn set_u16(&mut self, d: Datum, val: Option<u16>) {
        self.set_i16(d, val.map(|v| i16::from_ne_bytes(v.to_ne_bytes())));
    }

    pub fn get_i16(&self, d: Datum) -> Option<i16> {
        // TODO: refactor for DRY
        match d {
            Datum::Register(_) => self.get_i8(d).map(|x| x as i16),
            Datum::RegisterPair(x, y) => {
                if let Some(msb) = self.get_i8(Datum::Register(x)) {
                    if let Some(lsb) = self.get_i8(Datum::Register(y)) {
                        let high = u8::from_ne_bytes(msb.to_ne_bytes());
                        let low = u8::from_ne_bytes(lsb.to_ne_bytes());
                        return Some(i16::from_be_bytes([high, low]));
                    }
                }
                None
            }
            Datum::Imm8(d) => Some(d as i16),
            Datum::Imm16(d) => Some(d),
            Datum::Absolute(addr) => {
                if let Some(l) = self.heap.get(&addr) {
                    if let Some(h) = self.heap.get(&(addr + 1)) {
                        if let Some(low) = l {
                            if let Some(high) = h {
                                let high = u8::from_ne_bytes(high.to_ne_bytes());
                                let low = u8::from_ne_bytes(low.to_ne_bytes());
                                return Some(i16::from_be_bytes([high, low]));
                                // return Some((*high as i16 * 256) + *low as i16);
                            }
                        }
                    }
                }
                None
            }
            Datum::Zero => Some(0),
        }
    }

    pub fn get_u16(&self, d: Datum) -> Option<u16> {
        self.get_i16(d).map(|v| u16::from_ne_bytes(v.to_ne_bytes()))
    }

    pub fn set_i8(&mut self, d: Datum, val: Option<i8>) {
        match d {
            Datum::Register(register) => match register {
                R::A => {
                    self.accumulator = val;
                }
                R::B => {
                    self.reg_b = val;
                }
                R::C => {
                    self.reg_c = val;
                }
                R::D => {
                    self.reg_d = val;
                }
                R::E => {
                    self.reg_e = val;
                }
                R::H => {
                    self.reg_h = val;
                }
                R::L => {
                    self.reg_l = val;
                }
                R::H1 => {
                    self.reg_h1 = val;
                }
                R::L1 => {
                    self.reg_l1 = val;
                }
                R::Xl => {
                    self.xl = val;
                }
                R::Yl => {
                    self.yl = val;
                }
                R::Xh => {
                    self.xh = val;
                }
                R::Yh => {
                    self.yh = val;
                }
            },
            Datum::RegisterPair(h, l) => {
                self.set_i8(Datum::Register(l), val);
                self.set_i8(Datum::Register(h), Some(0));
            }
            Datum::Imm16(_) => {
                panic!()
            }
            Datum::Imm8(_) => {
                panic!()
            }
            Datum::Absolute(address) => {
                self.heap.insert(address, val);
            }
            Datum::Zero => {}
        }
    }
    pub fn set_i16(&mut self, d: Datum, val: Option<i16>) {
        let bytes = val.map(|v| v.to_be_bytes());
        let high = bytes.map(|v| i8::from_ne_bytes(v[0].to_ne_bytes()));
        let low = bytes.map(|v| i8::from_ne_bytes(v[1].to_ne_bytes()));

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
            Datum::Imm16(_) => {
                panic!()
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

fn random_shamt(width: u8) -> Datum {
    let s = (0..width).choose(&mut rand::thread_rng()).unwrap();
    Datum::Imm8(s.try_into().unwrap())
}

fn randomize_shamt(shamt: Datum, width: usize) -> Datum {
    match shamt {
        Datum::Imm8(n) => {
            if random() {
                Datum::Imm8(n.saturating_sub(1))
            } else {
                Datum::Imm8(n.saturating_add(1))
            }
        }
        anything_else => anything_else,
    }
}

fn flags_nz<T: num::PrimInt>(s: &mut State, a: Option<T>) {
    s.sign = a.map(|a| a.leading_zeros() == 0);
    s.zero = a.map(|a| a == T::zero());
}

fn rotate_right<T: num::PrimInt + Bits>(s: &mut State, a: Option<T>) -> Option<T> {
    let r = a.map(|a| a.unsigned_shr(1));
    let result = r
        .zip(s.carry)
        .map(|(v, c)| if c { v | <T as Bits>::msb() } else { v });
    s.carry = a.map(|a| a.trailing_zeros() == 0);
    flags_nz(s, result);
    result
}

fn arithmetic_shift_left<T: num::PrimInt + Bits>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| a.signed_shl(1));
    flags_nz(s, result);
    result
}

fn arithmetic_shift_right<T: num::PrimInt + Bits>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| a.signed_shr(1));
    s.carry = a.map(|a| a.trailing_zeros() == 0);
    flags_nz(s, result);
    result
}

fn logical_shift_right<T: num::PrimInt + Bits>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| a.unsigned_shr(1));
    flags_nz(s, result);
    result
}

fn standard_decrement<T: WrappingSub + num::PrimInt>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| a.wrapping_sub(&T::one()));
    flags_nz(s, result);
    result
}

fn standard_increment<T: WrappingAdd + num::PrimInt>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| a.wrapping_add(&T::one()));
    flags_nz(s, result);
    result
}

fn standard_negate<T: WrappingSub + num::PrimInt>(s: &mut State, a: Option<T>) -> Option<T> {
    let result = a.map(|a| T::zero().wrapping_sub(&a));
    flags_nz(s, result);
    result
}

fn standard_complement<T: WrappingSub + num::PrimInt>(s: &mut State, a: Option<T>) -> Option<T> {
    let minus_one = &T::zero().wrapping_sub(&T::one());
    let result = a.map(|a| a ^ *minus_one);
    flags_nz(s, result);
    result
}

fn standard_add<T: WrappingAdd + num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    b: Option<T>,
    carry: Option<bool>,
) -> Option<T> {
    let r = a.zip(b).zip(carry).map(|((a, b), c)| {
        a.wrapping_add(&b)
            .wrapping_add(if c { &T::one() } else { &T::zero() })
    });
    flags_nz(s, r);
    r
}

fn standard_sub<T: WrappingSub + num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    b: Option<T>,
    carry: Option<bool>,
) -> Option<T> {
    let r = a.zip(b).zip(carry).map(|((a, b), c)| {
        a.wrapping_sub(&b)
            .wrapping_sub(if c { &T::one() } else { &T::zero() })
    });
    flags_nz(s, r);
    r
}

fn standard_or<T: num::PrimInt>(s: &mut State, a: Option<T>, b: Option<T>) -> Option<T> {
    let r = a.zip(b).map(|(a, b)| a | b);
    flags_nz(s, r);
    r
}

fn standard_xor<T: num::PrimInt>(s: &mut State, a: Option<T>, b: Option<T>) -> Option<T> {
    let r = a.zip(b).map(|(a, b)| a ^ b);
    flags_nz(s, r);
    r
}

fn standard_bit_clear<T: num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    shamt: Option<usize>,
) -> Option<T> {
    let mask = shamt.map(|shamt| !(T::one().shr(shamt)));
    a.zip(mask).map(|(a, mask)| a & mask)
}

fn standard_bit_complement<T: num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    shamt: Option<usize>,
) -> Option<T> {
    let mask = shamt.map(|shamt| T::one().shr(shamt));
    a.zip(mask).map(|(a, mask)| a ^ mask)
}

fn standard_bit_set<T: num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    shamt: Option<usize>,
) -> Option<T> {
    let mask = shamt.map(|shamt| T::one().shr(shamt));
    a.zip(mask).map(|(a, mask)| a | mask)
}

#[cfg(test)]
mod fuzz {

    #[test]
    fn mos6502_fuzz_test() {
        use crate::machine::mos6502::tests::fuzz_test;
        fuzz_test();
    }
}
