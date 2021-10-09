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

pub fn equivalence(prog: &Vec<Instruction>, schema: &Schema, test_cases: &Vec<(Vec<i8>, Vec<i8>)>) -> bool {
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


pub fn exhaustive_search(found_it: &dyn Fn(&Vec<Instruction>) -> bool, instructions: Vec<Instruction>) {
    let instrs = instructions.iter().map(|i| (i.vectorize)(i)).flatten().collect();

	fn try_all(term: &dyn Fn(&Vec<Instruction>) -> bool, prog: Vec<Instruction>, instrs: &Vec<Instruction>, len: u32) -> bool {
		if len == 0 {
			/*
			println!("Trying:");
			for i in &prog {
				println!("{}", i);
			}
			*/
			return term(&prog);
		} else {
			for ins in instrs {
				let p = prog.iter().cloned().chain(vec![*ins]).collect();
				if try_all(term, p, &instrs, len-1) {
					return true;
				}
			}
			return false;
		}
	}

	let t: &dyn Fn(&Vec<Instruction>) -> bool = &|v| -> bool { found_it(v) };

	for i in 1..10 {
		println!("Trying programs of length {}.", i);
		if try_all(&t, Vec::new(), &instrs, i){
			return;
		}
	}

}
