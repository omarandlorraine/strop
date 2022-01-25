use std::collections::HashMap;
use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
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
    Pic12, Pic14, Pic16,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PreX86Variant {
    ZilogZ80,
    I8080,
    I8085,
    KR580VM1
}

#[derive(Clone, Copy, PartialEq)]
pub enum Machine {
    Mos6502(Mos6502Variant),
    Motorola6800(Motorola8BitVariant),
    Pic(PicVariant),
    PreX86(PreX86Variant)
}

#[derive(Clone, Copy)]
pub enum AddressingMode {
    Implicit,
    Immediate(i8),
    Absolute(u16),
    PicWF(bool, u16),
}

#[derive(Clone, Copy)]
pub struct Instruction {
    opname: &'static str,
    pub operation: Operation,
    pub randomize: fn(Machine, &mut Instruction),
    src: AddressingMode,
}


impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.operation)
    }
}

#[derive(Copy, Debug, Clone)]
pub enum Datum {
    A, B, X, Y,
    Immediate(i8),
    Absolute(u16),
    Zero,
}

impl Machine {
    pub fn register_by_name(self, name: &str) -> Datum {
        match self {
            Machine::Mos6502(_) => {
                match name {
                    "a" => { Datum::A }
                    "x" => { Datum::X }
                    "y" => { Datum::Y }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::Motorola6800(_) => {
                match name {
                    "a" => { Datum::A }
                    "b" => { Datum::B }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::Pic(_) => {
                match name {
                    "w" => { Datum::A }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::PreX86(_variant) => {
                // TODO: fill in for the other variants
                match name {
                    "a" => { Datum::A }
                    "b" => { Datum::B }
                    _ => { panic!("No such register as {}", name); }
                }
            }
        }
    }
}

pub fn bitwise_and(
    reg: Option<i8>,
    a: Option<i8>
) -> (
    Option<i8>,
    Option<bool>
) {
    if let Some(operand) = a {
        if let Some(r) = reg {
            return (Some(r & operand), Some(r & operand == 0));
        }
    }
    (None, None)
}

pub fn bitwise_xor(
    reg: Option<i8>,
    a: Option<i8>
) -> (
    Option<i8>,
    Option<bool>
) {
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
    if val.is_none() || carry.is_none() {
        (None, None)
    } else {
        let c = carry.unwrap();
        let v = val.unwrap();
        let high_bit_set = v & -128 != 0;
        let shifted = (v & 0x7f).rotate_left(1);
        (
            Some(if c { shifted + 1 } else { shifted }),
            Some(high_bit_set),
        )
    }
}

fn rotate_right_thru_carry(val: Option<i8>, carry: Option<bool>) -> (Option<i8>, Option<bool>) {
    if val.is_none() || carry.is_none() {
        (None, None)
    } else {
        let c = carry.unwrap();
        let v = val.unwrap();
        let low_bit_set = v & 1 != 0;
        let shifted = (v & 0x7f).rotate_right(1);
        (
            Some(if c { shifted | -128i8 } else { shifted }),
            Some(low_bit_set),
        )
    }
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
#[allow(non_camel_case_types)]
pub enum Operation {
    op_ror, op_rol, op_sta, op_lda, op_mov, op_com, op_stz, op_and,
    op_sec, op_clc, op_lsr,
    op_asl,
    op_daa,
    Decrement(Datum),
    Increment(Datum),
    Add(Datum, Datum),
    AddWithCarry(Datum, Datum),
    Move(Datum, Datum),
}

impl Instruction {
    pub fn inh(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            randomize: |_m, _i| (),
            src: AddressingMode::Implicit,
        }
    }

    pub fn imm(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            randomize: |_m, _i| (),
            src: AddressingMode::Immediate(0),
        }
    }

    pub fn abs(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            randomize: |_m, _i| (),
            src: AddressingMode::Absolute(0),
        }
    }

    pub fn new(
        randomize: for<'r> fn(Machine, &'r mut Instruction),
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname: "bonio", // needs to be removed
            operation,
            randomize,
            src: AddressingMode::Absolute(0), // needs to be removed
        }
    }

    pub fn pic_wf(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            randomize: |_m, _i| (),
            src: AddressingMode::PicWF(false, 0),
        }
    }

    fn get_datum(&self, m: &State) -> Option<i8> {
        match self.src {
            AddressingMode::Implicit => {
                panic!();
            }
            AddressingMode::Immediate(constant) => Some(constant),
            AddressingMode::Absolute(address) => {
                if let Some(x) = m.heap.get(&address) {
                    *x
                } else {
                    None
                }
            }
            AddressingMode::PicWF(_d, address) => {
                if let Some(x) = m.heap.get(&address) {
                    *x
                } else {
                    None
                }
            }
        }
    }

    fn write_datum(&self, m: &mut State, val: Option<i8>) {
        match self.src {
            AddressingMode::Implicit => {
                panic!();
            }
            AddressingMode::Immediate(_) => {
                panic!();
            }
            AddressingMode::Absolute(address) => {
                m.heap.insert(address, val);
            }
            AddressingMode::PicWF(f, address) => {
                if f {
                    m.heap.insert(address, val);
                }
                else {
                    m.accumulator = val;
                }
            }
        }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn operate(&self, s: &mut State) -> bool {
        match self.operation {
            Operation::Add(source, destination) => {
                let (result, c, z, n, o, h) = add_to_reg8(get(s, source), get(s, destination), Some(false));
                set(s, destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }
            Operation::AddWithCarry(source, destination) => {
                let (result, c, z, n, o, h) = add_to_reg8(get(s, source), get(s, destination), s.carry);
                set(s, destination, result);
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }
            Operation::Move(source, destination) => {
                set(s, destination, get(s, source));
                true
            }
            Operation::op_and => {
                let (result, z) = bitwise_and(s.accumulator, self.get_datum(s));
                s.accumulator = result;
                s.zero = z;
                true
            }

            Operation::op_asl => {
                let (val, c) = rotate_left_thru_carry(s.accumulator, Some(false));
                s.accumulator = val;
                s.carry = c;
                true
            }

            Operation::op_com => {
                let (result, z) = bitwise_xor(s.accumulator, Some(-1));
                s.accumulator = result;
                s.zero = z;
                true
            }

            Operation::op_clc => {
                s.carry = Some(false);
                true
            }

            Operation::op_daa => {
                s.accumulator = decimal_adjust(s.accumulator, s.carry, s.halfcarry);
                true
            }

            Operation::Increment(register) => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(get(s, register), Some(1), Some(false));
                set(s, register, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::Decrement(register) => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(get(s, register), Some(-1), Some(false));
                set(s, register, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_lda => {
                s.accumulator = self.get_datum(s);
                true
            }

            Operation::op_lsr => {
                let (val, c) = rotate_right_thru_carry(s.accumulator, Some(false));
                s.accumulator = val;
                s.carry = c;
                true
            }

            Operation::op_mov => {
                self.write_datum(s, self.get_datum(s));
                true
            }

            Operation::op_rol => {
                let (val, c) = rotate_left_thru_carry(s.accumulator, s.carry);
                s.accumulator = val;
                s.carry = c;
                true
            }

            Operation::op_ror => {
                let (val, c) = rotate_right_thru_carry(s.accumulator, s.carry);
                s.accumulator = val;
                s.carry = c;
                true
            }

            Operation::op_sec => {
                s.carry = Some(true);
                true
            }

            Operation::op_sta => {
                self.write_datum(s, s.accumulator);
                true
            }

            Operation::op_stz => {
                self.write_datum(s, Some(0));
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
    decimal: Option<bool>,
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
            decimal: None,
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
        Datum::Zero => {
        }
    }
}

pub fn get(state: &State, register: Datum) -> Option<i8> {
    match register {
        Datum::A => {
            state.accumulator
        }
        Datum::B => {
            state.reg_b
        }
        Datum::X => {
            state.x8
        }
        Datum::Y => {
            state.y8
        }
        Datum::Immediate(x) => {
            Some(x)
        }
        Datum::Absolute(address) => {
            if let Some(x) = state.heap.get(&address) {
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

fn random_immediate() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Immediate(*vs.choose(&mut rand::thread_rng()).unwrap())
}

fn random_absolute() -> Datum {
    let vs = vec![0, 1, 2, 3, 4];
    Datum::Absolute(*vs.choose(&mut rand::thread_rng()).unwrap())
}

pub fn motorola6800() -> Vec<Instruction> {
    fn random_add(_mach: Machine, instr: &mut Instruction) {
        fn random_accumulator() -> Datum {
            if random() {
                Datum::A
            } else {
                Datum::B
            }
        }

        fn random_source() -> Datum {
            // TODO: sort it out
            random_immediate() 
        }
        
        instr.operation = match rand::thread_rng().gen_range(0, 3) { 
            0 => { Operation::Add(Datum::B, Datum::A)},
            1 => { Operation::AddWithCarry(random_source(), random_accumulator())},
            _ => { Operation::Add(random_source(), random_accumulator())},
        };
    }

    fn random_t(_mach: Machine, instr: &mut Instruction) {
        instr.operation = match instr.operation {
            Operation::Move(Datum::A, Datum::B) => {Operation::Move(Datum::B, Datum::A)}
            Operation::Move(Datum::B, Datum::A) => {Operation::Move(Datum::A, Datum::B)}
            _ => {unreachable!()}
        }
    }
    
    vec![
        Instruction::new(random_add, Operation::Add(Datum::B, Datum::A)),
        Instruction::new(random_t, Operation::Move(Datum::B, Datum::A)),

        Instruction::inh("asla", Operation::op_asl),
        Instruction::inh("daa", Operation::op_daa),
        Instruction::inh("rol", Operation::op_rol),
        Instruction::inh("ror", Operation::op_ror),
        Instruction::inh("clc", Operation::op_clc),
        Instruction::inh("sec", Operation::op_sec),
    ]
}

pub fn mos6502() -> Vec<Instruction> {
    fn random_inc_dec(_mach: Machine, instr: &mut Instruction) {
        instr.operation = if random() {
            // pick between increment or decrement
            match instr.operation {
                Operation::Increment(x) => { Operation::Decrement(x) }
                Operation::Decrement(x) => { Operation::Increment(x) }
                _ => { panic!(); }
            }
        } else {
            // pick between X and Y
            // TODO: check if this machine supports inc/dec on accumulator
            match instr.operation {
                Operation::Increment(Datum::X) => { Operation::Decrement(Datum::Y) }
                Operation::Increment(Datum::Y) => { Operation::Decrement(Datum::X) }
                Operation::Decrement(Datum::X) => { Operation::Increment(Datum::Y) }
                Operation::Decrement(Datum::Y) => { Operation::Increment(Datum::X) }
                _ => { panic!(); }
            }
        }
    }

    fn random_add(_mach: Machine, instr: &mut Instruction) {
        fn random_source() -> Datum {
            if random() {
                random_immediate() 
            } else {
                random_absolute()
            }
        }
        
        instr.operation = Operation::AddWithCarry(random_source(), Datum::A);
    }

    fn random_t(_mach: Machine, instr: &mut Instruction) {
        instr.operation = match instr.operation {
            Operation::Move(Datum::A, Datum::X) => {
                if random() {
                    Operation::Move(Datum::A, Datum::Y)
                } else {
                    Operation::Move(Datum::X, Datum::A)
                }
            }
            Operation::Move(Datum::A, Datum::Y) => {
                if random() {
                    Operation::Move(Datum::A, Datum::X)
                } else {
                    Operation::Move(Datum::Y, Datum::A)
                }
            }
            Operation::Move(Datum::X, Datum::A) => {
                if random() {
                    Operation::Move(Datum::A, Datum::Y)
                } else {
                    Operation::Move(Datum::A, Datum::X)
                }
            }
            Operation::Move(Datum::Y, Datum::A) => {
                if random() {
                    Operation::Move(Datum::X, Datum::A)
                } else {
                    Operation::Move(Datum::A, Datum::X)
                }
            }
            _ => {unreachable!()}
        }
    }

    fn random_store(mach: Machine, instr: &mut Instruction) {
        fn random_register(z: bool) -> Datum {
            match rand::thread_rng().gen_range(0, if z { 4 } else { 3 }) {
                0 => { Datum::A }
                1 => { Datum::X }
                2 => { Datum::Y }
                _ => { Datum::Zero }
            }
        }

        instr.operation = match instr.operation {
            Operation::Move(src, dst) => {
                if random() {
                    Operation::Move(src, random_absolute())
                } else {
                    Operation::Move(random_register(mach == Machine::Mos6502(Mos6502Variant::Cmos)), dst)
                }
            }
            _ => { unreachable!() }
        };
    }

    fn random_load(_mach: Machine, instr: &mut Instruction) {
        fn random_register() -> Datum {
            match rand::thread_rng().gen_range(0, 3) {
                0 => { Datum::A }
                1 => { Datum::X }
                _ => { Datum::Y }
            }
        }
        fn random_source() -> Datum {
            if random() {
                random_absolute()
            } else {
                random_immediate()
            }
        }

        instr.operation = match instr.operation {
            Operation::Move(src, dst) => {
                if random() {
                    Operation::Move(src, random_register())
                } else {
                    Operation::Move(random_source(), dst)
                }
            }
            _ => { unreachable!() }
        };
    }

    vec![
        Instruction::new(random_inc_dec, Operation::Increment(Datum::X)),
        Instruction::new(random_add, Operation::AddWithCarry(Datum::Immediate(0), Datum::A)),
        Instruction::new(random_t, Operation::Move(Datum::A, Datum::X)),
        Instruction::new(random_store, Operation::Move(Datum::A, Datum::Absolute(0))),
        Instruction::new(random_load, Operation::Move(Datum::Immediate(0), Datum::A)),
        Instruction::inh("asl a", Operation::op_asl),
        Instruction::inh("rol", Operation::op_rol),
        Instruction::inh("ror", Operation::op_ror),
        Instruction::inh("lsr", Operation::op_lsr),
        Instruction::inh("clc", Operation::op_clc),
        Instruction::inh("sec", Operation::op_sec),
    ]
}

pub fn mos65c02() -> Vec<Instruction> {
    vec![
        Instruction::inh("stz", Operation::op_stz),
    ]
    .into_iter()
    .chain(mos6502())
    .collect()
}

pub fn z80() -> Vec<Instruction> {
    Vec::new()
}

pub fn i8080() -> Vec<Instruction> {
    Vec::new()
}

pub fn i8085() -> Vec<Instruction> {
    Vec::new()
}

pub fn pic12() -> Vec<Instruction> {
    fn random_inc_dec(_mach: Machine, instr: &mut Instruction) {
        // TODO: These instructions can optionally write to W instead of the F.
        instr.operation = match instr.operation {
            Operation::Decrement(x) => {
                if random() {
                    Operation::Increment(x)
                } else {
                    Operation::Decrement(random_absolute())
                }
            }
            Operation::Increment(x) => {
                if random() {
                    Operation::Decrement(x)
                } else {
                    Operation::Increment(random_absolute())
                }
            }
            _ => { unreachable!() }
        }
    }
    fn random_add(mach: Machine, instr: &mut Instruction) {
        fn addwf() -> Operation {
            if random() {
                Operation::Add(random_absolute(), Datum::A)
            } else {
                Operation::Add(Datum::A, random_absolute())
            }
        }
        fn addlw() -> Operation {
            Operation::Add(random_immediate(), Datum::A)
        }

        instr.operation = match mach {
            Machine::Pic(PicVariant::Pic12) => {
                addwf()
            }
            Machine::Pic(_) => {
                if random() { addlw() } else { addwf() }
            }
            _ => { unreachable!(); }
        }
    }


    vec![
        Instruction::new(random_add, Operation::Add(Datum::Absolute(0), Datum::A)),
        Instruction::new(random_inc_dec, Operation::Increment(Datum::X)),
        Instruction::imm("andlw", Operation::op_and),
        Instruction::pic_wf("andwf", Operation::op_and),
        // TODO: bcf bsf btfsc btfss (call) 
        Instruction::pic_wf("clr  ", Operation::op_stz),
        // TODO: (clrwdt)
        Instruction::abs("comf ", Operation::op_com),
        // TODO: decfsz (goto)
        // TODO: incfsz iorlw iorwf
        Instruction::abs("movf ", Operation::op_mov),
        Instruction::imm("movlw", Operation::op_lda),
        Instruction::pic_wf("movwf", Operation::op_sta),
        // TODO (nop) (option) (retlw)
        Instruction::abs("rlf  ", Operation::op_rol),
        Instruction::abs("rrf  ", Operation::op_ror),
        // TODO: (sleep) subwf swapf (tris) xorlw xorwf 
    ]
}
