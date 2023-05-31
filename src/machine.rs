use rand::seq::SliceRandom;
use std::collections::HashMap;
extern crate rand;

#[derive(Clone, Copy)]
pub enum AddressingMode {
    Implicit,
    Immediate(i8),
    Absolute(u16),
}

#[derive(Clone, Copy)]
pub struct Instruction {
    opname: &'static str,
    pub operation: fn(&Instruction, &mut State) -> bool,
    src: AddressingMode,
}

pub fn bitwise_and(reg: Option<i8>, a: Option<i8>) -> (Option<i8>, Option<bool>) {
    if let Some(operand) = a {
        if let Some(r) = reg {
            return (Some(r & operand), Some(r & operand == 0));
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

fn rotate_left_thru_carry(val: Option<i8>, carry: Option<bool>) -> (Option<i8>, Option<bool>) {
    if let Some((carry, val)) = carry.zip(val) {
        let c = carry;
        let v = val;
        let high_bit_set = v & -128 != 0;
        let shifted = (v & 0x7f).rotate_left(1);
        (
            Some(if c { shifted + 1 } else { shifted }),
            Some(high_bit_set),
        )
    } else {
        (None, None)
    }
}

fn rotate_right_thru_carry(val: Option<i8>, carry: Option<bool>) -> (Option<i8>, Option<bool>) {
    if let Some((carry, val)) = carry.zip(val) {
        let c = carry;
        let v = val;
        let low_bit_set = v & 1 != 0;
        let shifted = (v & 0x7f).rotate_right(1);
        (
            Some(if c { shifted | -128i8 } else { shifted }),
            Some(low_bit_set),
        )
    } else {
        (None, None)
    }
}

impl Instruction {
    pub fn inh(
        opname: &'static str,
        operation: for<'r, 's> fn(&'r Instruction, &'s mut State) -> bool,
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            src: AddressingMode::Implicit,
        }
    }

    pub fn imm(
        opname: &'static str,
        operation: for<'r, 's> fn(&'r Instruction, &'s mut State) -> bool,
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            src: AddressingMode::Immediate(0),
        }
    }

    pub fn abs(
        opname: &'static str,
        operation: for<'r, 's> fn(&'r Instruction, &'s mut State) -> bool,
    ) -> Instruction {
        Instruction {
            opname,
            operation,
            src: AddressingMode::Absolute(0),
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
        }
    }

    pub fn vectorize(&self, constants: &[i8], vars: &[u16]) -> Vec<Instruction> {
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
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn op_and(&self, s: &mut State) -> bool {
        let (result, z) = bitwise_and(s.accumulator, self.get_datum(s));
        s.accumulator = result;
        s.zero = z;
        true
    }

    fn op_asl(&self, s: &mut State) -> bool {
        let (val, c) = rotate_left_thru_carry(s.accumulator, Some(false));
        s.accumulator = val;
        s.carry = c;
        true
    }

    #[allow(clippy::many_single_char_names)]
    fn op_adc_dp(&self, s: &mut State) -> bool {
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

    fn op_clc(&self, s: &mut State) -> bool {
        s.carry = Some(false);
        true
    }

    fn op_dea(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.accumulator, Some(-1), Some(false));
        s.accumulator = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_dex(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(-1), Some(false));
        s.x8 = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_dey(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(-1), Some(false));
        s.y8 = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_ina(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.accumulator, Some(1), Some(false));
        s.accumulator = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_inx(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(1), Some(false));
        s.x8 = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_iny(&self, s: &mut State) -> bool {
        let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(1), Some(false));
        s.y8 = result;
        s.zero = z;
        s.sign = n;
        true
    }

    fn op_lda(&self, s: &mut State) -> bool {
        s.accumulator = self.get_datum(s);
        true
    }

    fn op_ldx(&self, s: &mut State) -> bool {
        s.x8 = self.get_datum(s);
        true
    }

    fn op_ldy(&self, s: &mut State) -> bool {
        s.y8 = self.get_datum(s);
        true
    }

    fn op_lsr(&self, s: &mut State) -> bool {
        let (val, c) = rotate_right_thru_carry(s.accumulator, Some(false));
        s.accumulator = val;
        s.carry = c;
        true
    }

    fn op_rol(&self, s: &mut State) -> bool {
        let (val, c) = rotate_left_thru_carry(s.accumulator, s.carry);
        s.accumulator = val;
        s.carry = c;
        true
    }

    fn op_ror(&self, s: &mut State) -> bool {
        let (val, c) = rotate_right_thru_carry(s.accumulator, s.carry);
        s.accumulator = val;
        s.carry = c;
        true
    }

    fn op_sec(&self, s: &mut State) -> bool {
        s.carry = Some(true);
        true
    }

    fn op_sta(&self, s: &mut State) -> bool {
        self.write_datum(s, s.accumulator);
        true
    }

    fn op_stx(&self, s: &mut State) -> bool {
        self.write_datum(s, s.x8);
        true
    }

    fn op_sty(&self, s: &mut State) -> bool {
        self.write_datum(s, s.y8);
        true
    }

    fn op_stz(&self, s: &mut State) -> bool {
        self.write_datum(s, Some(0));
        true
    }

    fn op_tax(&self, s: &mut State) -> bool {
        // TODO: This one definitely needs flags.
        s.x8 = s.accumulator;
        true
    }

    fn op_tay(&self, s: &mut State) -> bool {
        // TODO: This one definitely needs flags.
        s.y8 = s.accumulator;
        true
    }

    fn op_txa(&self, s: &mut State) -> bool {
        // TODO: This one definitely needs flags.
        s.accumulator = s.x8;
        true
    }

    fn op_tya(&self, s: &mut State) -> bool {
        // TODO: This one definitely needs flags.
        s.accumulator = s.y8;
        true
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
        }
    }
}

//#[derive(Copy, Clone)]
pub struct State {
    accumulator: Option<i8>,
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

pub fn set_a(state: &mut State, a: i8) {
    state.accumulator = Some(a);
}
pub fn get_a(state: &State) -> Option<i8> {
    state.accumulator
}

pub fn set_x(state: &mut State, x: i8) {
    state.x8 = Some(x);
}
pub fn get_x(state: &State) -> Option<i8> {
    state.x8
}

pub fn set_y(state: &mut State, y: i8) {
    state.y8 = Some(y);
}
pub fn get_y(state: &State) -> Option<i8> {
    state.y8
}

pub fn mos6502() -> Vec<Instruction> {
    vec![
        // TODO: Maybe we should have only one INC instruction, which can randomly go to either X or Y or the other possibilities.
        Instruction::inh("inx", Instruction::op_inx),
        Instruction::inh("iny", Instruction::op_iny),
        Instruction::inh("dex", Instruction::op_dex),
        Instruction::inh("dey", Instruction::op_dey),
        // TODO: Maybe we should have a single transfer instruction as well, which can go to one of tax txa tay tya txs tsx etc.
        Instruction::inh("tax", Instruction::op_tax),
        Instruction::inh("tay", Instruction::op_tay),
        Instruction::inh("txa", Instruction::op_txa),
        Instruction::inh("tya", Instruction::op_tya),
        Instruction::inh("asl a", Instruction::op_asl),
        Instruction::inh("rol", Instruction::op_rol),
        Instruction::inh("ror", Instruction::op_ror),
        Instruction::inh("lsr", Instruction::op_lsr),
        Instruction::inh("clc", Instruction::op_clc),
        Instruction::inh("sec", Instruction::op_sec),
        Instruction::imm("adc", Instruction::op_adc_dp),
        Instruction::imm("and", Instruction::op_and),
        Instruction::abs("adc", Instruction::op_adc_dp),
        Instruction::abs("lda", Instruction::op_lda),
        Instruction::abs("sta", Instruction::op_sta),
        Instruction::abs("ldx", Instruction::op_ldx),
        Instruction::abs("stx", Instruction::op_stx),
        Instruction::abs("ldy", Instruction::op_ldy),
        Instruction::abs("sty", Instruction::op_sty),
    ]
}

pub fn mos65c02() -> Vec<Instruction> {
    vec![
        Instruction::inh("ina", Instruction::op_ina),
        Instruction::inh("dea", Instruction::op_dea),
        Instruction::inh("stz", Instruction::op_stz),
    ]
    .into_iter()
    .chain(mos6502())
    .collect()
}
