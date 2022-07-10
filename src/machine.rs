use smallvec::SmallVec;
use std::borrow::Cow;

pub mod stm8;

#[derive(Clone, Copy)]
pub struct Machine {
    pub name: &'static str,
    reg_by_name: fn(&str) -> Result<Datum, &'static str>,
}

impl<State, Operand, OUD, IUD> std::fmt::Display for Instruction<'_, State, Operand, OUD, IUD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.disassemble)(f, self)
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

impl<State, Operand, OUD, IUD> Instruction<'_, State, Operand, OUD, IUD> {
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
pub struct Operation<'a, State, Operand, OUD, IUD> {
    pub disassemble: fn(&Instruction<'a, State, Operand, OUD, IUD>) -> Cow<'static, str>,
    pub mutate: fn(&mut Instruction<'a, State, Operand, OUD, IUD>) -> (),
    pub execute: fn(&Instruction<'a, State, Operand, OUD, IUD>, &mut State) -> u64,
    pub userdata: OUD,
}

#[derive(Clone)]
pub struct Instruction<'op, State, Operand, OUD, IUD> {
    pub operation: &'op Operation<'op, State, Operand, OUD, IUD>,
    pub operands: SmallVec<[Operand; 4]>,
    pub userdata: IUD,
}
