use crate::machine::mos6502::instr_length_6502;
use crate::machine::rand::prelude::SliceRandom;
use std::collections::HashMap;
extern crate rand;

mod m6800;
mod mos6502;
mod pic;
mod prex86;
mod stm8;

use crate::machine::m6800::instr_6800;
use crate::machine::mos6502::instr_6502;
use crate::machine::pic::instr_pic;
use crate::machine::prex86::instr_prex86;
use crate::machine::stm8::instr_stm8;

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
    KR580VM1,
    Sm83,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Machine {
    Mos6502(Mos6502Variant),
    Motorola6800(Motorola8BitVariant),
    Pic(PicVariant),
    PreX86(PreX86Variant),
    Stm8,
}

#[derive(Clone, Copy)]
pub struct Instruction {
    pub operation: Operation,
    randomizer: fn(Machine) -> Operation,
    disassemble: fn(Operation, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    machine: Machine,
}

#[derive(Clone, Copy, Debug)]
pub enum Width {
    Width8,
    Width16,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.disassemble)(self.operation, f)
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
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

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Datum {
    Register(R),
    RegisterPair(R, R),
    Imm8(i8),
    Absolute(u16),
    Zero,
}

impl Datum {
    pub fn width(&self) -> Width {
        match self {
            Self::Register(_) => Width::Width8,
            Self::RegisterPair(_, _) => Width::Width16,
            Self::Imm8(_) => Width::Width8,
            Self::Absolute(_) => Width::Width8,
            Self::Zero => Width::Width8,
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
                    if name == "h1" {
                        return Datum::Register(R::H1);
                    }
                    if name == "l1" {
                        return Datum::Register(R::L1);
                    }
                    if name == "h1l1" {
                        return Datum::RegisterPair(R::H1, R::L1);
                    }
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
            Machine::Stm8 => match name {
                "a" => Datum::Register(R::A),
                "x" => Datum::RegisterPair(R::Xh, R::Xl),
                "y" => Datum::RegisterPair(R::Yh, R::Yl),
                "xl" => Datum::Register(R::Xl),
                "yl" => Datum::Register(R::Yl),
                _ => {
                    panic!("No such register as {}", name);
                }
            },
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

pub fn bitwise_or(reg: Option<i8>, a: Option<i8>) -> (Option<i8>, Option<bool>) {
    if let Some(operand) = a {
        if let Some(r) = reg {
            return (Some(r | operand), Some(r | operand == 0));
        }
    }
    (None, None)
}

#[allow(clippy::many_single_char_names, clippy::type_complexity)]
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

#[allow(clippy::many_single_char_names, clippy::type_complexity)]
pub fn subtract_reg8(
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
                let v = operand.wrapping_sub(if c { 1 } else { 0 });
                let result = r.wrapping_sub(v);
                let z = result == 0;
                let c = r.checked_sub(v).is_none();
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

#[allow(clippy::many_single_char_names, clippy::type_complexity)]
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
            );
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn disasm(mach: Machine) {
        for _i in 0..5000 {
            let mut instr = new_instruction(mach);
            for _j in 0..50 {
                instr.randomize();
                let d = format!("{}", instr);
                if d[0..1] != "\t".to_owned() {
                    println!("No disassembly for instruction {}", d);
                    panic!();
                }
            }
        }
    }

    #[test]
    fn disassembler_6800() {
        disasm(Machine::Motorola6800(Motorola8BitVariant::Motorola6800));
        disasm(Machine::Motorola6800(Motorola8BitVariant::Motorola6801));
    }

    #[test]
    fn disassembler_prex86() {
        disasm(Machine::PreX86(PreX86Variant::ZilogZ80));
        disasm(Machine::PreX86(PreX86Variant::I8080));
        disasm(Machine::PreX86(PreX86Variant::Sm83));
        disasm(Machine::PreX86(PreX86Variant::KR580VM1));
    }

    #[test]
    fn disassembler_pic() {
        disasm(Machine::Pic(PicVariant::Pic12));
        disasm(Machine::Pic(PicVariant::Pic14));
        disasm(Machine::Pic(PicVariant::Pic16));
    }

    #[test]
    fn add_to_reg8_test() {
        assert_eq!(
            add_to_reg8(Some(3), Some(3), Some(false)),
            (
                Some(6),
                Some(false),
                Some(false),
                Some(false),
                Some(false),
                Some(false)
            )
        );
        assert_eq!(
            add_to_reg8(Some(127), Some(1), Some(false)),
            (
                Some(-128),
                Some(true),
                Some(false),
                Some(true),
                Some(true),
                Some(true)
            )
        );
        assert_eq!(
            add_to_reg8(None, Some(3), Some(false)),
            (None, None, None, None, None, None)
        );
    }
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
    Carry(bool),
    Bit(u16, u8, bool),
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum MonadicOperation {
    Complement,
    Decrement,
    Increment,
    Negate,
    // TODO: Move the shifts here.
}

#[derive(Clone, Debug, Copy)]
pub enum Operation {
    Monadic(Width, MonadicOperation, Datum, Datum),
    DecimalAdjustAccumulator,
    Add(Datum, Datum, bool),
    BitCompare(Datum, Datum),
    Compare(Datum, Datum),
    And(Datum, Datum),
    Or(Datum, Datum),
    Xor(Datum, Datum),
    ExclusiveOr(Datum, Datum),
    Move(Datum, Datum),
    Shift(ShiftType, Datum),
    Carry(bool),
    ComplementCarry,
    BitSet(Datum, u8),
    BitClear(Datum, u8),
    BitComplement(Datum, u8),
    BitCopyCarry(Datum, u8),
    Jump(Test, FlowControl),
}

impl Test {
    fn evaluate(&self, s: &State) -> Option<bool> {
        match self {
            Test::True => Some(true),
            Test::Carry(b) => s.carry.map(|carry| &carry == b),
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

impl Instruction {
    pub fn new(
        machine: Machine,
        randomizer: fn(Machine) -> Operation,
        disassemble: fn(Operation, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    ) -> Instruction {
        Instruction {
            machine,
            operation: randomizer(machine),
            disassemble,
            randomizer,
        }
    }

    pub fn randomize(&mut self) {
        self.operation = (self.randomizer)(self.machine);
    }

    pub fn len(&self) -> usize {
        match self.machine {
            Machine::Mos6502(_) => instr_length_6502(self.operation),
            // these architectures have fixed instruction widths
            Machine::Pic(_) => 1,
            // In case of unknown instruction length, assume 1 so that optimizer still works
            _ => 1,
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn operate(&self, s: &mut State) -> FlowControl {
        match self.operation {
            Operation::Add(source, destination, carry) => {
                if !carry {
                    s.carry = Some(false)
                };
                let (result, c, z, n, o, h) =
                    add_to_reg8(s.get_i8(source), s.get_i8(destination), s.carry);

                s.set_i8(destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
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
            Operation::Compare(source, destination) => {
                let (_result, c, z, n, _o, _h) =
                    subtract_reg8(s.get_i8(source), s.get_i8(destination), Some(false));
                s.sign = n;
                s.carry = c;
                s.zero = z;
                FlowControl::FallThrough
            }
            Operation::And(source, destination) => {
                let (result, z) = bitwise_and(s.get_i8(source), s.get_i8(destination));
                s.set_i8(destination, result);
                s.zero = z;
                FlowControl::FallThrough
            }
            Operation::Or(source, destination) => {
                let (result, z) = bitwise_or(s.get_i8(source), s.get_i8(destination));
                s.set_i8(destination, result);
                s.zero = z;
                FlowControl::FallThrough
            }
            Operation::Xor(source, destination) => {
                let (result, z) = bitwise_xor(s.get_i8(source), s.get_i8(destination));
                s.set_i8(destination, result);
                s.zero = z;
                FlowControl::FallThrough
            }
            Operation::ExclusiveOr(source, destination) => {
                let (result, z) = bitwise_xor(s.get_i8(source), s.get_i8(destination));
                s.set_i8(destination, result);
                s.zero = z;
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

            Operation::Increment(register) => {
                match register.width() {
                    Width::Width8 => {
                        let (result, _c, z, n, _o, _h) =
                            add_to_reg8(s.get_i8(register), Some(1), Some(false));
                        s.set_i8(register, result);
                        s.zero = z;
                        s.sign = n;
                    }
                    Width::Width16 => {
                        let (result, _c, z, n, _o, _h) =
                            add_to_reg16(s.get_i16(register), Some(1), Some(false));
                        s.set_i16(register, result);
                        s.zero = z;
                        s.sign = n;
                    }
                }
                FlowControl::FallThrough
            }

            Operation::Negate(register) => {
                let c = s.get_i8(register).map(|x| !x);
                s.set_i8(register, c);
                s.zero = c.map(|x| x == 0);
                FlowControl::FallThrough
            }

            Operation::Complement(register) => {
                let c = s.get_i8(register).map(|x| 0 - x);
                s.set_i8(register, c);
                s.zero = c.map(|x| x == 0);
                FlowControl::FallThrough
            }

            Operation::Decrement(register) => {
                let (result, _c, z, n, _o, _h) =
                    add_to_reg8(s.get_i8(register), Some(-1), Some(false));
                s.set_i8(register, result);
                s.zero = z;
                s.sign = n;
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

    pub fn get_i16(&self, d: Datum) -> Option<i16> {
        match d {
            Datum::Register(_) => self.get_i8(d).map(|x| x as i16),
            Datum::RegisterPair(x, y) => {
                if let Some(msb) = self.get_i8(Datum::Register(x)) {
                    if let Some(lsb) = self.get_i8(Datum::Register(y)) {
                        return Some(((msb as i16) << 8) | lsb as i16);
                    }
                }
                None
            }
            Datum::Imm8(d) => Some(d as i16),
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
            Datum::Zero => Some(0),
        }
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

pub fn new_instruction(mach: Machine) -> Instruction {
    match mach {
        Machine::Motorola6800(_) => instr_6800(mach),
        Machine::Mos6502(_) => instr_6502(mach),
        Machine::PreX86(_) => instr_prex86(mach),
        Machine::Pic(_) => instr_pic(mach),
        Machine::Stm8 => instr_stm8(mach),
    }
}
