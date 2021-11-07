use rand::Rng;
use rand::prelude::SliceRandom;
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
	if prog.iter().fold(true, |valid: bool, i| valid && (i.operation)(i, &mut s)) {
        Some(s)
    } else {
        None
    }
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

pub fn differance(prog: &Vec<Instruction>, schema: &Schema, test_cases: &Vec<(Vec<i8>, Vec<i8>)>) -> f64 {
    let mut ret: f64 = 0.0;
	for tc in test_cases {
		if let Some(state) = run_program(prog, schema, &tc.0) {
			for (func, val) in schema.live_out.iter().zip(&tc.1) {
                if let Some(v) = func(&state) {
                    let d: f64 = v.into();
                    let e: f64 = (*val).into();
                    ret += (d - e).abs();
                } else {
                    ret += 256.0; // the cost of an output variable that's never been written to
                }
			}
		} else {
			ret = ret + 1000.0; // what am I doing
		}
	}
    return ret;
}

fn disassemble(prog: &Vec<Instruction>) {
    for p in prog {
        println!("{}", p);
    }
}

pub fn stochastic_search(convergence: &dyn Fn(&Vec<Instruction>) -> f64, instructions: Vec<Instruction>, constants: Vec<i8>, vars: Vec<u16>) -> Vec<Instruction> {

    let mut prog: Vec<Instruction> = vec![];
    let mut current: Vec<Instruction> = vec![];

    while convergence(&current) > 0.01 {
        if convergence(&current) > convergence(&prog) {
            current = prog.clone();
            println!("\n\ncurrent:\n");
            disassemble(&current);
        }
        let mutate: usize = rand::thread_rng().gen_range(0, 3);
        match mutate {
            /* randomize an instruction 
             * (this could involve changing and operand, addressing mode, etc etc.
             */
            0 => {
                if prog.len() > 1 {
                    let offset: usize = rand::thread_rng().gen_range(0, prog.len());
                    prog[offset].randomize(&constants, &vars);
                }
            }
            /* delete an instruction */
            1 => {
                if prog.len() > 1 {
                    let offset: usize = rand::thread_rng().gen_range(0, prog.len());
                    prog.remove(offset);
                }
            }
            2 => {
                let offset: usize = if prog.len() > 0 {
                    rand::thread_rng().gen_range(0, prog.len())
                } else {
                    0
                };
                let instruction = instructions.choose(&mut rand::thread_rng()).unwrap();
                prog.insert(offset, *instruction);
                prog[offset].randomize(&constants, &vars);
            }
            _ => {
                panic!();
            }
        }
    }
    prog
}

pub fn exhaustive_search(found_it: &dyn Fn(&Vec<Instruction>) -> bool, instructions: Vec<Instruction>, constants: Vec<i8>, vars: Vec<u16>) {
    let instrs = instructions.iter().map(|i| i.vectorize(&constants, &vars)).flatten().collect();

	fn try_all(term: &dyn Fn(&Vec<Instruction>) -> bool, prog: &mut Vec<Instruction>, instrs: &Vec<Instruction>, len: u32) -> bool {
		if len == 0 {
			return term(&prog);
		} else {
			for ins in instrs {
				prog.push(*ins);
				if try_all(term, prog, &instrs, len-1) {
					return true;
				}
                prog.pop();
			}
			return false;
		}
	}

	let t: &dyn Fn(&Vec<Instruction>) -> bool = &|v| -> bool { found_it(v) };

	for i in 1..10 {
		println!("Trying programs of length {}.", i);
		if try_all(&t, &mut Vec::new(), &instrs, i){
			return;
		}
	}

}
