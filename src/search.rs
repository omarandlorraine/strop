use crate::machine::Instruction;
use crate::State;

pub struct SearchData<'a> {
	current_length: usize,
	cursor: usize,
	prog: Vec<Instruction>,
	pub constants: Vec<i8>,
	pub instrs: Vec<Instruction>,
	pub live_in: Vec<Box<dyn for<'r> Fn(&'r mut State, i8)>>,
	pub live_out: Vec<Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a >>,
	pub test_cases: Vec<(Vec<i8>, Vec<i8>)>,
}

pub fn empty_search_data() -> SearchData<'static>{
	SearchData {
		current_length: 0,
		cursor: 0,
		prog: Vec::new(),
		constants: Vec::new(),
		instrs: Vec::new(),
		live_in: Vec::new(),
		live_out: Vec::new(),
		test_cases: Vec::new(),
	}
}

fn run_program(d: &SearchData, inputs: &Vec<i8>) -> Option<State> {
	let mut s = State::new();
	
	for (func, val) in d.live_in.iter().zip(inputs) {
		(func)(&mut s, *val);
	}
	d.prog.iter().fold(Some(s), |mut state, i| (i.operation)(i, &mut state))
}

fn equivalence(d: &SearchData) -> bool {
	for tc in &d.test_cases {
		if let Some(state) = run_program(d, &tc.0) {
			for (func, val) in d.live_out.iter().zip(&tc.1) {
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

pub fn exhaustive_search(d: &mut SearchData) {
	// There's gotta be a less moronic way of doing this.

	println!("Trying programs of length 1.");
	for i in &d.instrs {
		d.prog = vec![*i];
		if equivalence(d) {
			disassemble(&d.prog);
			return;
		};
	}

	println!("Trying programs of length 2.");
	println!("Trying programs of length 3.");
	
}

pub fn dead_code_elimination(d: &mut SearchData) {
}
