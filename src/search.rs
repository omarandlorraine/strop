use crate::machine::Instruction;
use crate::State;

pub struct Schema<'a> {
	live_in: Vec<Box<dyn for<'r> Fn(&'r mut State, i8)>>,
	live_out: Vec<Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a >>,
}

impl<'a> Schema<'_> {
	pub fn new(live_in: Vec<Box<dyn for<'r> Fn(&'r mut State, i8)>>, live_out: Vec<Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a >>) -> Schema {
		Schema { live_in, live_out }
	}
}

fn run_program(prog: &Vec<Instruction>, schema: &Schema, inputs: &Vec<i8>) -> Option<State> {
	let mut s = State::new();
	
	for (func, val) in schema.live_in.iter().zip(inputs) {
		(func)(&mut s, *val);
	}
	prog.iter().fold(Some(s), |mut state, i| (i.operation)(i, &mut state))
}

fn equivalence(prog: &Vec<Instruction>, schema: &Schema, test_cases: &Vec<(Vec<i8>, Vec<i8>)>) -> bool {
	for tc in test_cases {
		if let Some(state) = run_program(prog, schema, &tc.0) {
			for (func, val) in schema.live_out.iter().zip(&tc.1) {
				let result = func(&state);
				if result != Some(*val) {
					return false;
				}
			}
		} else {
			return false;
		}
	}
	return true;
}

fn disassemble(p: &Vec<Instruction>) {
	println!("Disassembly:");
	for i in p {
		println!("\t{}", i);
	}
}

pub fn exhaustive_search(test_cases: &Vec<(Vec<i8>, Vec<i8>)>, schema: Schema, instrs: Vec<Instruction>) {
	// There's gotta be a less moronic way of doing this.

	println!("Trying programs of length 1.");
	for i in &instrs {
		let prog = vec![*i];
		if equivalence(&prog, &schema, test_cases) {
			disassemble(&prog);
			return;
		};
	}

	println!("Trying programs of length 2.");
	for i in &instrs {
		for j in &instrs {
			let prog = vec![*i, *j];
			if equivalence(&prog, &schema, test_cases) {
				disassemble(&prog);
				return;
			};
		}
	}

	println!("Trying programs of length 3.");
	for i in &instrs {
		for j in &instrs {
			for k in &instrs {
				let prog = vec![*i, *j, *k];
				if equivalence(&prog, &schema, test_cases) {
					disassemble(&prog);
					return;
				};
			}
		}
	}
	
}
