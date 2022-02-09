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

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn da_1operand(f: &mut std::fmt::Formatter<'_>, opcode: &str, operand: Datum) -> std::fmt::Result {
            match operand {
                Datum::A => { write!(f, "\t{} a", opcode) }
                Datum::B => { write!(f, "\t{} b", opcode) }
                Datum::X => { write!(f, "\t{} x", opcode) }
                Datum::Y => { write!(f, "\t{} y", opcode) }
                Datum::Zero => { write!(f, "\t{} zero", opcode) }
                Datum::Immediate(i) => { write!(f, "\t{} #{}", opcode, i) }
                Datum::Absolute(a) => { write!(f, "\t{} {}", opcode, a) }
            }
        }

        fn da_sh(f: &mut std::fmt::Formatter<'_>, shtype: ShiftType, d: Datum) -> std::fmt::Result {
            let opcode = match shtype {
                ShiftType::LeftRotateThroughCarry => { "rol" }
                ShiftType::RightRotateThroughCarry => { "ror" }
                ShiftType::LeftArithmetic => { "asl" }
                ShiftType::RightArithmetic => { "asr" }
            };
            da_1operand(f, opcode, d)
        }

        match (self.machine, self.operation) {
            (Machine::Mos6502(_), Operation::Move(Datum::A, Datum::X)) => { write!(f, "\ttax") }
            (Machine::Mos6502(_), Operation::Move(Datum::A, Datum::Y)) => { write!(f, "\ttay") }
            (Machine::Mos6502(_), Operation::Move(Datum::X, Datum::A)) => { write!(f, "\ttxa") }
            (Machine::Mos6502(_), Operation::Move(Datum::Y, Datum::A)) => { write!(f, "\ttya") }
            (Machine::Motorola6800(_), Operation::Move(Datum::B, Datum::A)) => { write!(f, "\ttba") } 
            (Machine::Motorola6800(_), Operation::Move(Datum::A, Datum::B)) => { write!(f, "\ttab") } 
            (Machine::Motorola6800(_), Operation::Add(Datum::B, Datum::A)) => { write!(f, "\taba") } 
            (_, Operation::Shift(shtype, datum)) => { da_sh(f, shtype, datum) }
            (Machine::Mos6502(_), Operation::Increment(Datum::A)) => { write!(f, "ina") }
            (Machine::Mos6502(_), Operation::Increment(Datum::X)) => { write!(f, "inx") }
            (Machine::Mos6502(_), Operation::Increment(Datum::Y)) => { write!(f, "iny") }
            (Machine::Mos6502(_), Operation::Decrement(Datum::A)) => { write!(f, "dea") }
            (Machine::Mos6502(_), Operation::Decrement(Datum::X)) => { write!(f, "dex") }
            (Machine::Mos6502(_), Operation::Decrement(Datum::Y)) => { write!(f, "dey") }
            (_, Operation::Increment(datum)) => { da_1operand(f, "inc", datum) }
            (_, Operation::Decrement(datum)) => { da_1operand(f, "dec", datum) }
            (Machine::Mos6502(_), Operation::Move(Datum::A, Datum::Absolute(a))) => { write!(f, "\tsta {}", a) }
            (Machine::Mos6502(_), Operation::Move(Datum::Absolute(a), Datum::A)) => { write!(f, "\tlda {}", a) }
            (_, Operation::AddWithCarry(Datum::Absolute(a), Datum::A)) => { write!(f, "\tadc {}", a) }
            (_, Operation::AddWithCarry(Datum::Zero, Datum::Absolute(a))) => { write!(f, "\tstz {}", a) }
            _ => { write!(f, "{:?}", self.operation) }
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Datum {
    A,
    B,
    X,
    Y,
    Immediate(i8),
    Absolute(u16),
    Zero,
}

impl Machine {
    pub fn register_by_name(self, name: &str) -> Datum {
        match self {
            Machine::Mos6502(_) => match name {
                "a" => Datum::A,
                "x" => Datum::X,
                "y" => Datum::Y,
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::Motorola6800(_) => match name {
                "a" => Datum::A,
                "b" => Datum::B,
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::Pic(_) => match name {
                "w" => Datum::A,
                _ => {
                    panic!("No such register as {}", name);
                }
            },
            Machine::PreX86(_variant) => {
                // TODO: fill in for the other variants
                match name {
                    "a" => Datum::A,
                    "b" => Datum::B,
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
        add_to_reg8(Some(3), 3),
        (Some(6), Some(false), Some(false), Some(false), Some(false))
    );
    assert_eq!(
        add_to_reg8(Some(127), 1),
        (Some(-128), Some(true), Some(false), Some(true), Some(false))
    );
    assert_eq!(add_to_reg8(None, 3), (None, None, None, None, None));
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
    Add(Datum, Datum),
    AddWithCarry(Datum, Datum),
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
            Operation::Add(source, destination) => {
                let (result, c, z, n, o, h) =
                    add_to_reg8(get(s, source), get(s, destination), Some(false));
                set(s, destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }
            Operation::AddWithCarry(source, destination) => {
                let (result, c, z, n, o, h) =
                    add_to_reg8(get(s, source), get(s, destination), s.carry);
                set(s, destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }
            Operation::And(source, destination) => {
                let (result, z) = bitwise_and(get(s, source), get(s, destination));
                set(s, destination, result);
                s.zero = z;
                true
            }
            Operation::Move(source, destination) => {
                set(s, destination, get(s, source));
                true
            }

            Operation::DecimalAdjustAccumulator => {
                s.accumulator = decimal_adjust(s.accumulator, s.carry, s.halfcarry);
                true
            }

            Operation::Increment(register) => {
                let (result, _c, z, n, _o, _h) =
                    add_to_reg8(get(s, register), Some(1), Some(false));
                set(s, register, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::Decrement(register) => {
                let (result, _c, z, n, _o, _h) =
                    add_to_reg8(get(s, register), Some(-1), Some(false));
                set(s, register, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::Shift(shtype, datum) => match shtype {
                ShiftType::LeftArithmetic => {
                    let (val, c) = rotate_left_thru_carry(get(s, datum), Some(false));
                    set(s, datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::RightArithmetic => {
                    let (val, c) = rotate_right_thru_carry(get(s, datum), Some(false));
                    set(s, datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::RightRotateThroughCarry => {
                    let (val, c) = rotate_right_thru_carry(get(s, datum), s.carry);
                    set(s, datum, val);
                    s.carry = c;
                    true
                }
                ShiftType::LeftRotateThroughCarry => {
                    let (val, c) = rotate_left_thru_carry(get(s, datum), s.carry);
                    set(s, datum, val);
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
    x8: Option<i8>,
    y8: Option<i8>,
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
            x8: None,
            y8: None,
            zero: None,
            carry: None,
            sign: None,
            overflow: None,
            halfcarry: None,
            heap: HashMap::new(),
        }
    }
}

pub fn set(state: &mut State, register: Datum, val: Option<i8>) {
    match register {
        Datum::A => {
            state.accumulator = val;
        }
        Datum::B => {
            state.reg_b = val;
        }
        Datum::X => {
            state.x8 = val;
        }
        Datum::Y => {
            state.y8 = val;
        }
        Datum::Immediate(_) => {
            panic!();
        }
        Datum::Absolute(address) => {
            state.heap.insert(address, val);
        }
        Datum::Zero => {}
    }
}

pub fn get(state: &State, register: Datum) -> Option<i8> {
    match register {
        Datum::A => state.accumulator,
        Datum::B => state.reg_b,
        Datum::X => state.x8,
        Datum::Y => state.y8,
        Datum::Immediate(x) => Some(x),
        Datum::Absolute(address) => {
            if let Some(x) = state.heap.get(&address) {
                *x
            } else {
                None
            }
        }
        Datum::Zero => Some(0),
    }
}

fn random_immediate() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Immediate(*vs.choose(&mut rand::thread_rng()).unwrap())
}

fn random_absolute() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Absolute(*vs.choose(&mut rand::thread_rng()).unwrap())
}

pub fn instr_prex86(_mach: Machine) -> Instruction {
    unimplemented!();
}

fn random_accumulator_6800() -> Datum {
    if random() {
        Datum::A
    } else {
        Datum::B
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
    if dst == Datum::A && random() {
        return Operation::Add(Datum::B, dst); // ABA
    }
    let src = random_source_6800();
    if random() {
        Operation::Add(src, dst) // ADDA and ADDB
    } else {
        Operation::AddWithCarry(src, dst) // ADCA and ADCB
    }
}

fn transfers_6800(_mach: Machine) -> Operation {
    if random() {
        Operation::Move(Datum::A, Datum::B)
    } else {
        Operation::Move(Datum::B, Datum::A)
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
        0 => {Datum::X}
        1 => {Datum::Y}
        _ => {Datum::A}
    };
    if random() {
        Operation::Increment(reg)
    } else {
        Operation::Decrement(reg)
    }
}

fn add_6502(_mach: Machine) -> Operation {
    Operation::AddWithCarry(random_source_6502(), Datum::A)
}

fn transfers_6502(_mach: Machine) -> Operation {
    let reg = if random() {
        Datum::X
    } else {
        Datum::Y
    };
    if random() {
        Operation::Move(Datum::A, reg)
    } else {
        Operation::Move(reg, Datum::A)
    }
}

fn loadstore_6502(mach: Machine) -> Operation {
    // TODO: STZ operation for CMOS varieties
    let addr = random_absolute();
    let reg = match rand::thread_rng().gen_range(0, if mach == Machine::Mos6502(Mos6502Variant::Cmos) { 4 } else { 3 }) {
        0 => Datum::A,
        1 => Datum::X,
        2 => Datum::Y,
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
        Datum::A
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
        Datum::A
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
    if dst == Datum::A && mach != Machine::Pic(PicVariant::Pic12) && random() {
        // This is an immediate add. Not available on PIC12.
        Operation::Add(random_immediate(), Datum::A) // addlw k
    } else if random() {
        Operation::Add(random_absolute(), Datum::A) // addwf f
    } else {
        Operation::Add(Datum::A, random_absolute()) // addwf f,d
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
    if dst == Datum::A && random() {
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
        0 => { Operation::Move(Datum::Zero, random_accumulator_or_absolute()) } // clrw and clrf f
        1 => { Operation::Move(random_accumulator_or_absolute(), Datum::A) }    // movf f
        2 => { Operation::Move(random_immediate(), Datum::A) }                  // movlw k
        _ => { Operation::Move(Datum::A, random_accumulator_or_absolute()) }    // movwf f
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
