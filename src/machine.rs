use crate::machine::rand::prelude::SliceRandom;
use std::collections::HashMap;
extern crate num;
extern crate rand;
use num::traits::{WrappingAdd, WrappingSub};
use std::convert::TryInto;

pub mod mos6502;
pub mod stm8;
pub mod x80;

use crate::machine::mos6502::{MOS6502, MOS65C02};
use crate::machine::x80::KR580VM1;

#[derive(Clone, Copy)]
pub struct Machine {
    pub name: &'static str,
    reg_by_name: fn(&str) -> Result<Datum, &'static str>,
}

pub const MACHINES: [Machine; 3] = [KR580VM1, MOS6502, MOS65C02];

pub trait Instruction: std::fmt::Display + Clone + Sized {
    fn randomize(&mut self);
    fn len(&self) -> usize;
    fn operate(&self, s: &mut State) -> FlowControl;
    fn random() -> Self
    where
        Self: Sized;
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
    fn shift_left(self, bit_in: bool) -> (bool, Self);
    fn shift_right(self, bit_in: bool) -> (bool, Self);
}

impl Swap for i8 {
    fn complement(self) -> i8 {
        self ^ -1
    }
    fn swap(self) -> i8 {
        self.rotate_right(4)
    }
    fn shift_left(self, bit_in: bool) -> (bool, Self) {
        let high_bit_set = self & -128 != 0;
        let shifted = (self & 0x7f).rotate_left(1);
        (high_bit_set, if bit_in { shifted + 1 } else { shifted })
    }
    fn shift_right(self, bit_in: bool) -> (bool, Self) {
        let low_bit_set = self & 1 != 0;
        let shifted = (self & -2).rotate_right(1);
        (low_bit_set, if bit_in { shifted | -128i8 } else { shifted })
    }
}

impl Swap for i16 {
    fn complement(self) -> i16 {
        self ^ -1
    }
    fn swap(self) -> i16 {
        self.swap_bytes()
    }
    fn shift_left(self, bit_in: bool) -> (bool, Self) {
        let high_bit_set = self & -32768i16 != 0;
        // the -2 is to mask out the lowest bit so it doesn't wrap to the top
        let shifted = (self & 0x7fff).rotate_left(1);
        (high_bit_set, if bit_in { shifted + 1 } else { shifted })
    }
    fn shift_right(self, bit_in: bool) -> (bool, Self) {
        let low_bit_set = self & 1 != 0;
        // the -2 is to mask out the lowest bit so it doesn't wrap to the top
        let shifted = (self & -2).rotate_right(1);
        (
            low_bit_set,
            if bit_in { shifted | -32768i16 } else { shifted },
        )
    }
}

impl Machine {
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
pub enum FlowControl {
    FallThrough,
    Forward(u8),
    Backward(u8),
    Invalid,
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

impl MonadicOperation {
    fn flags_sign_zero<T>(&self, s: &mut State, v: Option<T>)
    where
        T: num::PrimInt + std::iter::Sum + WrappingAdd + WrappingSub + Swap,
    {
        s.sign = v.map(|v| v.leading_zeros() == 0);
        s.zero = v.map(|v| v == T::zero());
    }

    fn evaluate<T>(&self, s: &mut State, v: Option<T>) -> Option<T>
    where
        T: num::PrimInt + std::iter::Sum + WrappingAdd + WrappingSub + Swap,
    {
        match self {
            Self::LeftShiftArithmetic => {
                let result = v.map(|v| v.shift_left(false).1);
                self.flags_sign_zero(s, result);
                s.carry = v.map(|v| v.leading_zeros() == 0);
                result
            }
            Self::RightShiftArithmetic => v.map(|v| v.shift_right(v < T::zero()).1),
            Self::RightShiftLogical => {
                let result = v.map(|v| v.shift_right(false).1);
                self.flags_sign_zero(s, result);
                s.carry = v.map(|v| v.trailing_zeros() == 0);
                result
            }
            Self::RotateLeftThruCarry => {
                let result = v
                    .map(|v| s.carry.map(|c| v.shift_left(c).1))
                    .unwrap_or(None);
                self.flags_sign_zero(s, result);
                s.carry = v.map(|v| v.leading_zeros() == 0);
                result
            }
            Self::RotateRightThruCarry => {
                let result = v
                    .map(|v| s.carry.map(|c| v.shift_right(c).1))
                    .unwrap_or(None);
                self.flags_sign_zero(s, result);
                s.carry = v.map(|v| v.trailing_zeros() == 0);
                result
            }
            Self::RotateLeftThruAccumulator => {
                panic!("no standard implementation of RotateLeftThruAccumulator")
            }
            Self::RotateRightThruAccumulator => {
                panic!("no standard implementation of RotateRightThruAccumulator")
            }
            Self::Complement => v.map(|v| v.complement()),
            Self::Negate => v.map(|v| T::zero().wrapping_sub(&v)),
            Self::Increment => v.map(|v| v.wrapping_add(&T::one())),
            Self::Decrement => v.map(|v| v.wrapping_sub(&T::one())),
            Self::Swap => v.map(|v| v.swap()),
        }
    }
}

impl DyadicOperation {
    fn evaluate<T>(&self, s: &mut State, a: Option<T>, b: Option<T>) -> Option<T>
    where
        T: num::PrimInt + std::iter::Sum + WrappingAdd + WrappingSub,
    {
        let (zero, one) = (&T::zero(), &T::one());
        if let (Some(a), Some(b)) = (a, b) {
            match self {
                Self::Add => Some(a.wrapping_add(&b)),
                Self::AddWithCarry => {
                    if let Some(c) = s.carry {
                        let result = a.wrapping_add(&b).wrapping_add(if c { one } else { zero });
                        if let Some(r) = a.checked_add(&b) {
                            s.carry = Some(r.checked_add(if c { one } else { zero }).is_none());
                        } else {
                            s.carry = Some(true);
                        }
                        let a_sign = a.leading_zeros() == 0;
                        let b_sign = b.leading_zeros() == 0;
                        let r_sign = result.leading_zeros() == 0;
                        s.zero = Some(result == *zero);
                        s.sign = Some(r_sign);
                        s.overflow =
                            Some((a_sign && b_sign && !r_sign) || (!a_sign && !b_sign && r_sign));
                        Some(result)
                    } else {
                        s.carry = None;
                        s.zero = None;
                        s.sign = None;
                        s.overflow = None;
                        None
                    }
                }
                Self::And => {
                    let result = a & b;
                    s.sign = Some(result.leading_zeros() == 0);
                    s.zero = Some(result == *zero);
                    Some(result)
                }
                Self::ExclusiveOr => {
                    let result = a ^ b;
                    s.sign = Some(result.leading_zeros() == 0);
                    s.zero = Some(result == *zero);
                    Some(result)
                }
                Self::Or => {
                    let result = a | b;
                    s.sign = Some(result.leading_zeros() == 0);
                    s.zero = Some(result == *zero);
                    Some(result)
                }
                Self::Subtract => Some(a.wrapping_sub(&b)),
                Self::SubtractWithCarry => {
                    if let Some(c) = s.carry {
                        let result = a.wrapping_sub(&b).wrapping_sub(if c { zero } else { one });
                        if let Some(r) = a.checked_sub(&b) {
                            s.carry = Some(r.checked_sub(if c { one } else { zero }).is_none());
                        } else {
                            s.carry = Some(true);
                        }
                        let a_sign = a.leading_zeros() == 0;
                        let b_sign = b.leading_zeros() == 0;
                        let r_sign = result.leading_zeros() == 0;
                        s.zero = Some(result == *zero);
                        s.sign = Some(r_sign);
                        s.overflow =
                            Some((a_sign && b_sign && !r_sign) || (!a_sign && !b_sign && r_sign));
                        Some(result)
                    } else {
                        s.carry = None;
                        s.zero = None;
                        s.sign = None;
                        s.overflow = None;
                        None
                    }
                }
                Self::SubtractWithBorrow => s
                    .carry
                    .map(|c| a.wrapping_sub(&b).wrapping_sub(if c { one } else { zero })),
                Self::Divide => {
                    unimplemented!("No standard implementation of DyadicOperation::Divide")
                }
                Self::Multiply => {
                    unimplemented!("No standard implementation of DyadicOperation::Multiply")
                }
            }
        } else {
            None
        }
    }
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
    Jump(Test, FlowControl),
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

impl FlowControl {
    pub fn newpc(self, pc: usize) -> Option<usize> {
        match self {
            FlowControl::FallThrough => Some(pc + 1),
            FlowControl::Forward(offs) => pc.checked_add(offs.into()),
            FlowControl::Backward(offs) => pc.checked_sub(offs.into()),
            FlowControl::Invalid => None,
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

#[cfg(test)]
mod fuzz {

    #[test]
    fn mos6502_fuzz_test() {
        use crate::machine::mos6502::tests::fuzz_test;
        fuzz_test();
    }
}
