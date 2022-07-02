use crate::machine::rand::prelude::SliceRandom;
use std::collections::HashMap;
extern crate num;
extern crate rand;
use crate::machine::rand::prelude::IteratorRandom;
use num::traits::{WrappingAdd, WrappingSub};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::convert::TryInto;

pub mod stm8;

use crate::machine::stm8::STM8;

use rand::random;

#[derive(Clone, Copy)]
pub struct Machine {
    pub name: &'static str,
    random_insn: fn() -> Instruction<'static, State, Operand, OUD, IUD>,
    reg_by_name: fn(&str) -> Result<Datum, &'static str>,
}

pub const MACHINES: [Machine; 1] = [STM8];

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

impl std::fmt::Display for Instruction<'_, State, Operand, OUD, IUD> {
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
            Self::Nothing => panic!(),
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
    pub fn new_instruction<'a>(self) -> Instruction<'a, State, Operand, OUD, IUD> {
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

impl Instruction<'_, State, Operand, OUD, IUD> {
    pub fn randomize(&mut self) {
        (self.randomizer)(self);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn operate(&self, s: &mut State) {
        (self.implementation)(self, s)
    }
}

#[derive(Clone)]
pub struct Operation<State, Operand, OUD, IUD> {
    pub disassemble: fn(&Instruction<State, Operand, OUD, IUD>) -> Cow<'static, str>,
    pub mutate: fn(&mut Instruction<State, Operand, OUD, IUD>) -> (),
    pub execute: fn(&Instruction<State, Operand, OUD, IUD>, &mut State) -> u64,
    pub userdata: OUD,
}

#[derive(Clone)]
pub struct Instruction<'op, State, Operand, OUD, IUD> {
    pub operation: &'op Operation<State, Operand, OUD, IUD>,
    pub operands: SmallVec<[Operand; 4]>,
    pub userdata: IUD,
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
            Datum::Nothing => panic!(),
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
            Datum::Nothing => panic!(),
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
            Datum::Nothing => {
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
            Datum::Nothing => panic!(),
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

fn rotate_left<T: num::PrimInt + Bits>(s: &mut State, a: Option<T>) -> Option<T> {
    let r = a.map(|a| a.unsigned_shl(1));
    let result = r
        .zip(s.carry)
        .map(|(v, c)| if c { v | T::one() } else { v });
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
    let one = T::one();
    let zero = T::zero();
    if let Some(((a, b), c)) = a.zip(b).zip(s.carry) {
        let result = a
            .wrapping_add(&b)
            .wrapping_add(if c { &one } else { &zero });
        if let Some(r) = a.checked_add(&b) {
            s.carry = Some(r.checked_add(if c { &one } else { &zero }).is_none());
        } else {
            s.carry = Some(true);
        }
        let a_sign = a.leading_zeros() == 0;
        let b_sign = b.leading_zeros() == 0;
        let r_sign = result.leading_zeros() == 0;
        s.zero = Some(result == zero);
        s.sign = Some(r_sign);
        s.overflow = Some((a_sign && b_sign && !r_sign) || (!a_sign && !b_sign && r_sign));
        Some(result)
    } else {
        s.carry = None;
        s.zero = None;
        s.sign = None;
        s.overflow = None;
        None
    }
}

fn standard_subtract<T: WrappingSub + num::PrimInt>(
    s: &mut State,
    a: Option<T>,
    b: Option<T>,
    carry: Option<bool>,
) -> Option<T> {
    let one = T::one();
    let zero = T::zero();
    if let Some(((a, b), c)) = a.zip(b).zip(s.carry) {
        let result = a
            .wrapping_sub(&b)
            .wrapping_sub(if c { &zero } else { &one });
        if let Some(r) = a.checked_sub(&b) {
            s.carry = Some(r.checked_sub(if c { &one } else { &zero }).is_none());
        } else {
            s.carry = Some(true);
        }
        let a_sign = a.leading_zeros() == 0;
        let b_sign = b.leading_zeros() == 0;
        let r_sign = result.leading_zeros() == 0;
        s.zero = Some(result == zero);
        s.sign = Some(r_sign);
        s.overflow = Some((a_sign && b_sign && !r_sign) || (!a_sign && !b_sign && r_sign));
        return Some(result);
    } else {
        s.carry = None;
        s.zero = None;
        s.sign = None;
        s.overflow = None;
        return None;
    }
}

fn standard_and<T: num::PrimInt>(s: &mut State, a: Option<T>, b: Option<T>) -> Option<T> {
    let r = a.zip(b).map(|(a, b)| a & b);
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

fn standard_bit_clear<T: num::PrimInt>(a: Option<T>, shamt: Option<usize>) -> Option<T> {
    let mask = shamt.map(|shamt| !(T::one().shr(shamt)));
    a.zip(mask).map(|(a, mask)| a & mask)
}

fn standard_bit_complement<T: num::PrimInt>(a: Option<T>, shamt: Option<usize>) -> Option<T> {
    let mask = shamt.map(|shamt| T::one().shr(shamt));
    a.zip(mask).map(|(a, mask)| a ^ mask)
}

fn standard_bit_set<T: num::PrimInt>(a: Option<T>, shamt: Option<usize>) -> Option<T> {
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
