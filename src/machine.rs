extern crate rand;

#[derive(Clone, Copy)]
pub enum AddressingMode {
	Implicit,
	Accumulator,
	Immediate(i8),
	//ZeroPage,
	//ZeroPageX,
	//ZeroPageY,
	//Relative,
	//Absolute,
	//AbsoluteX,
	//AbsoluteY,
	//Indirect,
	//IndexedIndirect,
	//IndirectIndexed,
}

#[derive(Clone, Copy)]
pub struct Instruction {
	opname: &'static str,
	pub operation: fn(&Instruction, &mut Option<State>) -> Option<State>,
	addressingmode: AddressingMode,
}

pub fn add_to_reg8(reg: Option<i8>, a: Option<i8>) -> (Option<i8>, Option<bool>, Option<bool>, Option<bool>, Option<bool>, Option<bool>) {
	// The return values are the result of the addition, then the flags, carry, zerro, sign, overflow, half-carry.
	if let Some(v) = a {
	if let Some(r) = reg {
		let result = r.wrapping_add(v);
		let z = if result == 0 { true } else { false };
		let c = if r.checked_add(v).is_none() { true } else { false };
		let n = if result < 0 { true } else { false };
		let o = (r < 0 && v < 0 && result >= 0) || (r > 0 && v > 0 && result <= 0);
		let h = ((r ^ v ^ result ) & 0x10) == 0x10;
		(Some(result), Some(c), Some(z), Some(n), Some(o), Some(h))
	} else {
		(None, None, None, None, None, None)
	}
	} else {
		(None, None, None, None, None, None)
	}
}

fn decimal_adjust(accumulator: Option<i8>, carry: Option<bool>, halfcarry: Option<bool>) -> Option<i8> {
	fn nybble(val: i8, flag: Option<bool>) -> Option<i8> {
		if val & 0x0f > 0x09 {
			return Some(0x06);
		}
		if flag.is_none() {
			return None;
		}
		if flag.unwrap_or(false) {
			return Some(0x06);
		}
		return Some(0);
	}

	if let Some(a) = accumulator {
		if let Some(right) = nybble(a, halfcarry) {
			let ar = a + right;
			if let Some(left) = nybble(ar >> 4, carry) {
				Some(ar + (left << 4))
			} else {
				None
			}
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
		(Some(if c { shifted + 1 } else { shifted }), Some(high_bit_set))
	}
}

#[test]
fn add_to_reg8_test() {
	assert_eq!(add_to_reg8(Some(3), 3), (Some(6), Some(false), Some(false), Some(false), Some(false)));
	assert_eq!(add_to_reg8(Some(127), 1), (Some(-128), Some(true), Some(false), Some(true), Some(false)));
	assert_eq!(add_to_reg8(None, 3), (None, None, None, None, None));
}

impl Instruction {
	pub fn new(opname: &'static str, operation: for<'r, 's> fn(&'r Instruction, &'s mut Option<State>) -> Option<State>, randomisers: Vec<fn(&mut Instruction)>) -> Instruction {
		Instruction{opname, operation, addressingmode: AddressingMode::Implicit}
	}

	pub fn random_implied(&mut self) {
		self.addressingmode = AddressingMode::Implicit;
	}

	pub fn random_immediate(&mut self) {
		self.addressingmode = AddressingMode::Immediate(rand::random());
	}

	fn operation_aba(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, s.reg_b);
			Some(State {
				accumulator: result,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: n,
				carry: c,
				zero: z,
				decimal: s.decimal,
				overflow: o,
				halfcarry: h
			})
		} else {
			None
		}
	}

	fn operation_add(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, c, z, n, o, h) = add_to_reg8(s.accumulator, self.get_datum(s));
			Some(State {
				accumulator: result,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: n,
				carry: c,
				zero: z,
				decimal: s.decimal,
				overflow: o,
				halfcarry: h
			})
		} else {
			None
		}
	}

	fn operation_clc(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: s.sign,
				carry: Some(false),
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_dex(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(-1));
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: result,
				y8: s.y8,
				carry: s.carry,
				zero: z,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: n,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_dey(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(-1));
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: result,
				carry: s.carry,
				zero: z,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: n,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_inx(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, _c, z, n, _o, _h) = add_to_reg8(s.x8, Some(1));
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: result,
				y8: s.y8,
				carry: s.carry,
				zero: z,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: n,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_iny(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			let (result, _c, z, n, _o, _h) = add_to_reg8(s.y8, Some(1));
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: result,
				carry: s.carry,
				zero: z,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: n,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_rol(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s{
			let (val, c) = rotate_left_thru_carry(s.accumulator, s.carry);
			Some(State {
				accumulator: val,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: s.sign,
				carry: c,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_sec(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: s.sign,
				carry: Some(true),
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_tab(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.accumulator,
				x8: s.x8,
				y8: s.y8,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_tax(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.accumulator,
				y8: s.y8,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_tay(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.accumulator,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.accumulator,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_tba(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.reg_b,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_txa(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.x8,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}

	fn operation_tya(&self, s: &mut Option<State>) -> Option<State> {
		if let Some(s) = s {
			Some(State {
				accumulator: s.y8,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				carry: s.carry,
				zero: s.zero,
				decimal: s.decimal,
				overflow: s.overflow,
				sign: s.sign,
				halfcarry: s.halfcarry
			})
		} else {
			None
		}
	}
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.addressingmode {
            AddressingMode::Implicit => {
                write!(f, "\t{}", self.opname)
            }
            AddressingMode::Accumulator => {
                write!(f, "\t{} a", self.opname)
            }
            AddressingMode::Immediate(constant) => {
                write!(f, "\t{} #{}", self.opname, constant)
            }
        }
    }
}

impl Iterator for Instruction {
    type Item = Instruction;

    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        todo!()
    }
}

#[derive(Copy, Clone)]
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
	halfcarry: Option<bool>
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
		}
	}
}

pub fn set_a(state: &mut State, a: i8) { state.accumulator = Some(a); }
pub fn get_a(state: &State) -> Option<i8> { state.accumulator } 

pub fn set_b(state: &mut State, b: i8) { state.reg_b = Some(b); }
pub fn get_b(state: &State) -> Option<i8> { state.reg_b } 

pub fn set_x(state: &mut State, x: i8) { state.x8 = Some(x); }
pub fn get_x(state: &State) -> Option<i8> { state.x8 } 

pub fn set_y(state: &mut State, y: i8) { state.y8 = Some(y); }
pub fn get_y(state: &State) -> Option<i8> { state.y8 } 

pub fn motorola6800() -> Vec<Instruction> {
	vec![
	Instruction::new("aba", Instruction::operation_aba, vec![Instruction::random_implied]),
	Instruction::new("tab", Instruction::operation_tab, vec![Instruction::random_implied]),
	Instruction::new("tba", Instruction::operation_tba, vec![Instruction::random_implied]),
	Instruction::new("rol", Instruction::operation_rol, vec![Instruction::random_implied]),
	Instruction::new("clc", Instruction::operation_clc, vec![Instruction::random_implied]),
	]
}

pub fn mos6502() -> Vec<Instruction> {
	vec![
	// TODO: Maybe we should have only one INC instruction, which can randomly go to either X or Y or the other possibilities.
	Instruction::new("inx", Instruction::operation_inx, vec![Instruction::random_implied]),
	Instruction::new("iny", Instruction::operation_iny, vec![Instruction::random_implied]),
	Instruction::new("dex", Instruction::operation_dex, vec![Instruction::random_implied]),
	Instruction::new("dey", Instruction::operation_dey, vec![Instruction::random_implied]),

	// TODO: Maybe we should have a single transfer instruction as well, which can go to one of tax txa tay tya txs tsx etc.
	Instruction::new("tax", Instruction::operation_tax, vec![Instruction::random_implied]),
	Instruction::new("tay", Instruction::operation_tay, vec![Instruction::random_implied]),
	Instruction::new("txa", Instruction::operation_txa, vec![Instruction::random_implied]),
	Instruction::new("tya", Instruction::operation_tya, vec![Instruction::random_implied]),

	Instruction::new("add", Instruction::operation_add, vec![Instruction::random_immediate]),
	]
}

pub fn mos65c02() -> Vec<Instruction> {
	mos6502()
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
	Vec::new()
}

pub fn pic14() -> Vec<Instruction> {
	Vec::new()
}

pub fn pic16() -> Vec<Instruction> {
	Vec::new()
}
