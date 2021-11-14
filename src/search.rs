use crate::machine::Instruction;
use crate::State;
use rand::prelude::SliceRandom;
use rand::Rng;

pub struct Schema<'a> {
    live_in: Vec<Box<dyn for<'r> Fn(&'r mut State, i8)>>,
    live_out: Vec<Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a>>,
}

impl<'a> Schema<'_> {
    pub fn new(
        live_in: Vec<Box<dyn for<'r> Fn(&'r mut State, i8)>>,
        live_out: Vec<Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a>>,
    ) -> Schema {
        Schema { live_in, live_out }
    }
}

fn run_program(prog: &Vec<Instruction>, schema: &Schema, inputs: &Vec<i8>) -> Option<State> {
    let mut s = State::new();

    for (func, val) in schema.live_in.iter().zip(inputs) {
        (func)(&mut s, *val);
    }
    if prog
        .iter()
        .fold(true, |valid: bool, i| valid && (i.operation)(i, &mut s))
    {
        Some(s)
    } else {
        None
    }
}

pub fn equivalence(
    prog: &Vec<Instruction>,
    schema: &Schema,
    test_cases: &Vec<(Vec<i8>, Vec<i8>)>,
) -> bool {
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
    true
}

pub fn differance(
    prog: &Vec<Instruction>,
    schema: &Schema,
    test_cases: &Vec<(Vec<i8>, Vec<i8>)>,
) -> f64 {
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
            ret += 1000.0; // what am I doing
        }
    }
    ret
}

fn disassemble(prog: &Vec<Instruction>) {
    for p in prog {
        println!("{}", p);
    }
}

fn cost(prog: &Vec<Instruction>) -> f64 {
    /* quick and simple cost function,
     * number of instructions in the program.
     * Not really a bad thing to minimise for.
     */
    prog.len() as f64
}

fn mutate_delete(prog: &mut Vec<Instruction>) {
    if prog.len() > 1 {
        let offset: usize = rand::thread_rng().gen_range(0, prog.len());
        prog.remove(offset);
    }
}

fn mutate_insert(prog: &mut Vec<Instruction>, instructions: &Vec<Instruction>, constants: &Vec<i8>, vars: &Vec<u16>) {
    let offset: usize = if prog.len() > 0 {
        rand::thread_rng().gen_range(0, prog.len())
    } else {
        0
    };
    let instruction = instructions.choose(&mut rand::thread_rng()).unwrap();
    prog.insert(offset, *instruction);
    prog[offset].randomize(&constants, &vars);
}

fn mutate(
    prog: &mut Vec<Instruction>,
    instructions: &Vec<Instruction>,
    constants: &Vec<i8>,
    vars: &Vec<u16>,
) {
    let mutate: usize = rand::thread_rng().gen_range(0, 3);
    match mutate {
        /* randomize an instruction
         * (this could involve changing an operand, addressing mode, etc etc.
         */
        0 => {
            if prog.len() > 1 {
                let offset: usize = rand::thread_rng().gen_range(0, prog.len());
                prog[offset].randomize(&constants, &vars);
            }
        }
        /* delete an instruction */
        1 => {
            mutate_delete(prog);
        }
        /* insert a new instruction */
        2 => {
            mutate_insert(prog, instructions, constants, vars);
        }
        /* Pick two instructions and swap them round */
        3 => {
            let offset_a: usize = rand::thread_rng().gen_range(0, prog.len());
            let offset_b: usize = rand::thread_rng().gen_range(0, prog.len());
            let ins_a = prog[offset_a];
            let ins_b = prog[offset_b];
            prog[offset_a] = ins_b;
            prog[offset_b] = ins_a;
        }
        _ => {
            panic!();
        }
    }
}

pub fn dead_code_elimination(
    convergence: &dyn Fn(&Vec<Instruction>) -> f64,
    prog: &Vec<Instruction>,
) -> Vec<Instruction> {
    let mut better = prog.clone();

    for _m in 1..1000 {
        let mut putative = prog.clone();
        for _n in 1..100 {
            mutate_delete(&mut putative);
            if convergence(&better) >= convergence(&putative) {
                better = putative.clone();
            } else {
                break;
            }
        }
    }
    better
}

pub fn stochastic_search(
    convergence: &dyn Fn(&Vec<Instruction>) -> f64,
    instructions: &Vec<Instruction>,
    constants: &Vec<i8>,
    vars: &Vec<u16>,
) -> Vec<Instruction> {

    // Initial population of a bajillion stupid programs
    // which are of course unlikely to be any good
    let mut population: Vec<(f64, Vec<Instruction>)> = vec![];
    for _i in 1..1000 {
        let mut program: Vec<Instruction> = vec![];
        for _j in 1..50 {
            mutate_insert(&mut program, instructions, constants, vars);
        }
        population.push((convergence(&program.clone()), program));
    }

    loop {

        // Now get the best one of all and print it out
        if let Some(best) = population.iter().min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN")) {
            println!("\n\n{}", best.0);
            disassemble(&best.1);
        } else {
            println!("none found :(");
            return vec![];
        }

        // compute the average fitness
        let avg_fit: f64 = Iterator::sum::<f64>(population.iter().map(|s| s.0)) / population.len() as f64;
        println!("avg_fit {}", avg_fit);

        // get rid of all lower-than-average specimens
        population.retain(|s| s.0 < avg_fit);
        println!("population size {}", population.len());

        // If the population size is not too great,
        // then the rest of the population may now make babies.
        if population.len() < 99 {
            let mut next_generation: Vec<(f64, Vec<Instruction>)> = vec![];
            for parent in &population {
                let mut child = parent.1.clone();
                dead_code_elimination(convergence, &child);

                // let's say, I don't know, fifty kids each.
                for _i in 0..50 {
                    mutate(&mut child, instructions, constants, vars);
                    let fitness = convergence(&child);

                    // but only the ones that are actually better make it
                    if fitness < avg_fit {
                        next_generation.push((fitness, child.clone()));
                    }
                }
            }
            population.append(&mut next_generation);
        }

    }
    //dead_code_elimination(convergence, &prog)
    //prog
}

pub fn exhaustive_search(
    found_it: &dyn Fn(&Vec<Instruction>) -> bool,
    instructions: Vec<Instruction>,
    constants: Vec<i8>,
    vars: Vec<u16>,
) {
    let instrs = instructions
        .iter()
        .map(|i| i.vectorize(&constants, &vars))
        .flatten()
        .collect();

    fn try_all(
        term: &dyn Fn(&Vec<Instruction>) -> bool,
        prog: &mut Vec<Instruction>,
        instrs: &Vec<Instruction>,
        len: u32,
    ) -> bool {
        if len == 0 {
            term(prog)
        } else {
            for ins in instrs {
                prog.push(*ins);
                if try_all(term, prog, &instrs, len - 1) {
                    return true;
                }
                prog.pop();
            }
            false
        }
    }

    let t: &dyn Fn(&Vec<Instruction>) -> bool = &|v| -> bool { found_it(v) };

    for i in 1..10 {
        println!("Trying programs of length {}.", i);
        if try_all(&t, &mut Vec::new(), &instrs, i) {
            return;
        }
    }
}
