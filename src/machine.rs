use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::machine::rand::Rng;
use std::collections::HashMap;
extern crate rand;

#[derive(Clone, Copy)]
pub enum Mos6502Variant {
    Nmos,
    Ricoh2a03,
    Cmos,
    IllegalInstructions,
}

#[derive(Clone, Copy)]
pub enum Motorola8BitVariant {
    Motorola6800,
    Motorola6801,
}

#[derive(Clone, Copy)]
pub enum PicVariant {
    Pic12, Pic14, Pic16,
}

#[derive(Clone, Copy)]
pub enum PreX86Variant {
    ZilogZ80,
    I8080,
    I8085,
    KR580VM1
}

#[derive(Clone, Copy)]
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
    src: AddressingMode,
}

#[derive(Copy, Clone)]
pub enum Register {
    A, B, X, Y
}

impl Machine {
    pub fn register_by_name(self, name: &str) -> Register {
        match self {
            Machine::Mos6502(_) => {
                match name {
                    "a" => { Register::A }
                    "x" => { Register::X }
                    "y" => { Register::Y }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::Motorola6800(_) => {
                match name {
                    "a" => { Register::A }
                    "b" => { Register::B }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::Pic(_) => {
                match name {
                    "w" => { Register::A }
                    _ => { panic!("No such register as {}", name); }
                }
            }
            Machine::PreX86(_variant) => {
                // TODO: fill in for the other variants
                match name {
                    "a" => { Register::A }
                    "b" => { Register::B }
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
    return (None, None);
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
    return (None, None);
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
            nybble(ar >> 4, carry).map(|left| ar.wrapping_add((left << 4)))
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

#[derive(Clone, Copy)]
pub enum Operation {
    op_add, op_ror, op_rol, op_sta, op_lda, op_mov, op_inc, op_dec, op_com, op_stz, op_and,
    op_dea, op_ina, op_sty, op_ldy, op_ldx, op_stx, op_sec, op_clc, op_lsr, op_adc_dp,
    op_asl, op_tya, op_txa, op_tay, op_tax, op_dey, op_dex, op_inx, op_iny, op_tba, op_tab,
    op_daa, op_adc, op_aba,
}

impl Instruction {
    pub fn inh(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
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
            src: AddressingMode::Absolute(0),
        }
    }

    pub fn pic_wf(
        opname: &'static str,
        operation: Operation
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            src: AddressingMode::PicWF(false, 0),
        }
    }

    pub fn randomize(&mut self, constants: &Vec<i8>, vars: &Vec<u16>) {

        fn address(vars: &Vec<u16>) -> u16 {
            if let Some(r) = vars.choose(&mut rand::thread_rng()) {
                // If there's any variables, then pick one.
                *r
            } else {
                // Otherwise pick any random address. (this is unlikely to be any good)
                rand::random()
            }
        }

        match self.src {
            AddressingMode::Implicit => {
                self.src = AddressingMode::Implicit;
            }
            AddressingMode::Immediate(_) => {
                if let Some(r) = constants.choose(&mut rand::thread_rng()) {
                    // If there's any constants, then pick one.
                    self.src = AddressingMode::Immediate(*r);
                } else {
                    // Otherwise pick any i8.
                    self.src = AddressingMode::Immediate(rand::random());
                }
            }
            AddressingMode::Absolute(_) => {
                self.src = AddressingMode::Absolute(address(vars));
            }
            AddressingMode::PicWF(_, _) => {
                let mut rng = thread_rng();
                self.src = AddressingMode::PicWF(rng.gen_bool(0.5), address(vars));
            }
        }
    }

    pub fn vectorize(&self, constants: &Vec<i8>, vars: &Vec<u16>) -> Vec<Instruction> {
        match self.src {
            AddressingMode::Implicit => {
                vec![*self]
            }
            AddressingMode::Immediate(_) => (*constants
                .iter()
                .map(|c| Instruction {
                    opname: self.opname,
                    operation: self.operation,
                    src: AddressingMode::Immediate(*c),
                })
                .collect::<Vec<Instruction>>())
                .to_vec(),
            AddressingMode::Absolute(_) => (*vars
                .iter()
                .map(|c| Instruction {
                    opname: self.opname,
                    operation: self.operation,
                    src: AddressingMode::Absolute(*c),
                })
                .collect::<Vec<Instruction>>())
                .to_vec(),
            AddressingMode::PicWF(_, _) => (*vars
                .iter()
                .map(|c| Instruction {
                    opname: self.opname,
                    operation: self.operation,
                    src: AddressingMode::Absolute(*c),
                })
                .collect::<Vec<Instruction>>())
                .to_vec(),
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
            Operation::op_aba => {
                // It looks like the ABA instruction of the 6800 doesn't use the carry flag.
                let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, s.reg_b, Some(false));
                s.accumulator = result;
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }

            Operation::op_add => {
                let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, self.get_datum(s), Some(false));
                s.accumulator = result;
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
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

            Operation::op_adc => {
                let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, self.get_datum(s), s.carry);
                s.accumulator = result;
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
                true
            }

            Operation::op_adc_dp => {
                // TODO: Check decimal flag here.
                let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, self.get_datum(s), s.carry);
                s.accumulator = result;
                s.sign = n;
                s.carry = c;
                s.zero = z;
                s.overflow = o;
                s.halfcarry = h;
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

            Operation::op_dea => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.accumulator, Some(-1), Some(false));
                s.accumulator = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_dec => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(self.get_datum(s), Some(-1), Some(false));
                s.x8 = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_dex => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(-1), Some(false));
                s.x8 = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_dey => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(-1), Some(false));
                s.y8 = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_ina => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.accumulator, Some(1), Some(false));
                s.accumulator = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_inc => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(self.get_datum(s), Some(1), Some(false));
                self.write_datum(s, result);
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_inx => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(1), Some(false));
                s.x8 = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_iny => {
                let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(1), Some(false));
                s.y8 = result;
                s.zero = z;
                s.sign = n;
                true
            }

            Operation::op_lda => {
                s.accumulator = self.get_datum(s);
                true
            }

            Operation::op_ldx => {
                s.x8 = self.get_datum(s);
                true
            }

            Operation::op_ldy => {
                s.y8 = self.get_datum(s);
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

            Operation::op_stx => {
                self.write_datum(s, s.x8);
                true
            }

            Operation::op_sty => {
                self.write_datum(s, s.y8);
                true
            }

            Operation::op_stz => {
                self.write_datum(s, Some(0));
                true
            }

            Operation::op_tab => {
                // TODO: We need to check if this instruction affects flags or not,
                // I feel like this is an oversight
                s.reg_b = s.accumulator;
                true
            }

            Operation::op_tax => {
                // TODO: This one definitely needs flags.
                s.x8 = s.accumulator;
                true
            }

            Operation::op_tay => {
                // TODO: This one definitely needs flags.
                s.y8 = s.accumulator;
                true
            }

            Operation::op_tba => {
                // TODO: We need to check if this instruction affects flags or not,
                // I feel like this is an oversight
                s.accumulator = s.reg_b;
                true
            }

            Operation::op_txa => {
                // TODO: This one definitely needs flags.
                s.accumulator = s.x8;
                true
            }

            Operation::op_tya => {
                // TODO: This one definitely needs flags.
                s.accumulator = s.y8;
                true
            }
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.src {
            AddressingMode::Implicit => {
                write!(f, "\t{}", self.opname)
            }
            AddressingMode::Immediate(constant) => {
                write!(f, "\t{} #{}", self.opname, constant)
            }
            AddressingMode::Absolute(address) => {
                write!(f, "\t{} {}", self.opname, address)
            }
            AddressingMode::PicWF(d, address) => {
                if d {
                    write!(f, "\t{} {},d", self.opname, address)
                } else {
                    write!(f, "\t{} {}", self.opname, address)
                }
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

pub fn set(state: &mut State, register: Register, val: i8) {
    match register {
        Register::A => {
            state.accumulator = Some(val);
        }
        Register::B => {
            state.reg_b = Some(val);
        }
        Register::X => {
            state.x8 = Some(val);
        }
        Register::Y => {
            state.y8 = Some(val);
        }
    }
}

pub fn get(state: &State, register: Register) -> Option<i8> {
    match register {
        Register::A => {
            state.accumulator
        }
        Register::B => {
            state.reg_b
        }
        Register::X => {
            state.x8
        }
        Register::Y => {
            state.y8
        }
    }
}

pub fn motorola6800() -> Vec<Instruction> {
    vec![
        Instruction::inh("aba", Operation::op_aba),
        Instruction::imm("add", Operation::op_add),
        Instruction::imm("adc", Operation::op_adc),
        Instruction::inh("asla", Operation::op_asl),
        Instruction::inh("daa", Operation::op_daa),
        Instruction::inh("tab", Operation::op_tab),
        Instruction::inh("tba", Operation::op_tba),
        Instruction::inh("rol", Operation::op_rol),
        Instruction::inh("ror", Operation::op_ror),
        Instruction::inh("clc", Operation::op_clc),
        Instruction::inh("sec", Operation::op_sec),
    ]
}

pub fn mos6502() -> Vec<Instruction> {
    vec![
        // TODO: Maybe we should have only one INC instruction, which can randomly go to either X or Y or the other possibilities.
        Instruction::inh("inx", Operation::op_inx),
        Instruction::inh("iny", Operation::op_iny),
        Instruction::inh("dex", Operation::op_dex),
        Instruction::inh("dey", Operation::op_dey),
        // TODO: Maybe we should have a single transfer instruction as well, which can go to one of tax txa tay tya txs tsx etc.
        Instruction::inh("tax", Operation::op_tax),
        Instruction::inh("tay", Operation::op_tay),
        Instruction::inh("txa", Operation::op_txa),
        Instruction::inh("tya", Operation::op_tya),
        Instruction::inh("asl a", Operation::op_asl),
        Instruction::inh("rol", Operation::op_rol),
        Instruction::inh("ror", Operation::op_ror),
        Instruction::inh("lsr", Operation::op_lsr),
        Instruction::inh("clc", Operation::op_clc),
        Instruction::inh("sec", Operation::op_sec),
        Instruction::imm("adc", Operation::op_adc_dp),
        Instruction::abs("adc", Operation::op_adc_dp),
        Instruction::abs("lda", Operation::op_lda),
        Instruction::abs("sta", Operation::op_sta),
        Instruction::abs("ldx", Operation::op_ldx),
        Instruction::abs("stx", Operation::op_stx),
        Instruction::abs("ldy", Operation::op_ldy),
        Instruction::abs("sty", Operation::op_sty),
    ]
}

pub fn mos65c02() -> Vec<Instruction> {
    vec![
        Instruction::inh("ina", Operation::op_ina),
        Instruction::inh("dea", Operation::op_dea),
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

pub fn iz80() -> Vec<Instruction> {
    Vec::new()
}

pub fn pic12() -> Vec<Instruction> {
    vec![
        Instruction::pic_wf("addwf", Operation::op_add),
        Instruction::imm("andlw", Operation::op_and),
        Instruction::pic_wf("andwf", Operation::op_and),
        // TODO: bcf bsf btfsc btfss (call) 
        Instruction::pic_wf("clr  ", Operation::op_stz),
        // TODO: (clrwdt)
        Instruction::abs("comf ", Operation::op_com),
        Instruction::abs("decf ", Operation::op_dec),
        // TODO: decfsz (goto)
        Instruction::abs("incf ", Operation::op_inc),
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

pub fn pic14() -> Vec<Instruction> {
    // From what I can tell from reading datasheets 41291D.pdf and 41239a.pdf,
    // these four instructions are the ones that exist in PIC14 and not in
    // PIC12.
    // There also are instructions that exist in PIC12 and not in PIC14. They
    // write to registers which are memory mapped in PIC14. The instructions
    // include tris and option.
    vec![
        Instruction::imm("addlw", Operation::op_add),
        // TODO: (retfie) (return)
        // TODO: sublw
    ]
    .into_iter()
    .chain(pic12())
    .collect()
}

pub fn pic16() -> Vec<Instruction> {
    pic14()
}
