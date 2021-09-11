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

pub fn add_to_reg8(reg: Option<i8>, c: i8) -> (Option<i8>, Option<bool>, Option<bool>, Option<bool>) {
	// The return values are the result of the addition, the carry flag, the zero flag, the sign flag.
	if let Some(r) = reg {
		let result = r.wrapping_add(c);
		let z = if result == 0 { true } else { false };
		let c = if r.checked_add(c).is_none() { true } else { false };
		let n = if result < 0 { true } else { false };
		(Some(result), Some(c), Some(z), Some(n))
	} else {
		(None, None, None, None)
	}
}

#[test]
fn add_to_reg8_test() {
	assert_eq!(add_to_reg8(Some(3), 3), (Some(6), Some(false), Some(false), Some(false)));
	assert_eq!(add_to_reg8(Some(127), 1), (Some(-128), Some(true), Some(false), Some(true)));
	assert_eq!(add_to_reg8(None, 3), (None, None, None, None));
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
			let a: Option<i8> = 
				if s.carry.is_none() || s.reg_b.is_none() || s.accumulator.is_none() {
					None
				} else {
					Some(s.reg_b.unwrap() + s.accumulator.unwrap() + if s.carry.unwrap() { 1 } else { 0 })
				};
			Some(State {
				accumulator: a,
				reg_b: s.reg_b,
				x8: s.x8,
				y8: s.y8,
				sign: s.sign, // TODO
				carry: s.carry, // TODO
				zero: s.zero, // TODO
				decimal: s.decimal,
				overflow: s.overflow,
			})
		} else {
			None
		}
	}

		fn operation_dex(&self, s: &mut Option<State>) -> Option<State> {
			if let Some(s) = s {
				let (result, _c, z, n) = add_to_reg8(s.x8, -1);
				Some(State {
					accumulator: s.accumulator,
					reg_b: s.reg_b,
					x8: result,
					y8: s.y8,
					carry: s.carry,
					zero: z,
					decimal: s.decimal,
					overflow: s.overflow,
					sign: n
				})
			} else {
				None
			}
		}

		fn operation_dey(&self, s: &mut Option<State>) -> Option<State> {
			if let Some(s) = s {
				let (result, _c, z, n) = add_to_reg8(s.y8, -1);
				Some(State {
					accumulator: s.accumulator,
					reg_b: s.reg_b,
					x8: s.x8,
					y8: result,
					carry: s.carry,
					zero: z,
					decimal: s.decimal,
					overflow: s.overflow,
					sign: n
				})
			} else {
				None
			}
		}

		fn operation_inx(&self, s: &mut Option<State>) -> Option<State> {
			if let Some(s) = s {
				let (result, _c, z, n) = add_to_reg8(s.x8, 1);
				Some(State {
					accumulator: s.accumulator,
					reg_b: s.reg_b,
					x8: result,
					y8: s.y8,
					carry: s.carry,
					zero: z,
					decimal: s.decimal,
					overflow: s.overflow,
					sign: n
				})
			} else {
				None
			}
		}

		fn operation_iny(&self, s: &mut Option<State>) -> Option<State> {
			if let Some(s) = s {
				let (result, _c, z, n) = add_to_reg8(s.y8, 1);
				Some(State {
					accumulator: s.accumulator,
					reg_b: s.reg_b,
					x8: s.x8,
					y8: result,
					carry: s.carry,
					zero: z,
					decimal: s.decimal,
					overflow: s.overflow,
					sign: n
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
					sign: s.sign
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
					sign: s.sign
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
					sign: s.sign
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
					sign: s.sign
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
					sign: s.sign
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
					sign: s.sign
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

fn get_datum(m: State, i: Instruction) -> Option<i8> {
	match i.addressingmode {
		AddressingMode::Implicit => {
			panic!();
		}
		AddressingMode::Accumulator => {
			m.accumulator
		}
		AddressingMode::Immediate(constant) => {
			Some(constant)
		}
	}
}

fn decimal_adjust(a: i8) -> i8 {
	let bcd1: i8 = if (a & 0x0f) as u8 > 0x09 {
		0x06
	} else {
		0x00
	};

	let bcd2: i8 = if (a.wrapping_add(bcd1) as u8 & 0xf0) as u8 > 0x90 {
		0x60
	} else {
		0x00
	};

	bcd2
}

#[derive(Copy, Clone)]
pub struct State {
	accumulator: Option<i8>,
	reg_b: Option<i8>,
	pub x8: Option<i8>,
	y8: Option<i8>,
	zero: Option<bool>,
	carry: Option<bool>,
	sign: Option<bool>,
	decimal: Option<bool>,
	overflow: Option<bool>
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
			overflow: None
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
