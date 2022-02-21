use crate::machine::Instruction;
use crate::{State, Test, TestRun, Machine};
use rand::Rng;
use std::ops::{Index, IndexMut};
use crate::machine::new_instruction;

#[derive(Clone)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

struct BasicBlockSpawn {
    parent: BasicBlock,
    mutant: BasicBlock,
    ncount: usize,
    mach: Machine
}

impl Iterator for BasicBlockSpawn {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ncount == 0 {
            self.mutant = self.parent.clone();
            self.ncount = 100;
        }
        self.ncount -= 1;
        mutate(
            &mut self.mutant,
            self.mach,
        );
        Some(self.mutant.clone())
    }
}

impl BasicBlock {
    fn initial_guess(
        mach: Machine,
        max_size: i32,
    ) -> BasicBlock {
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        for _i in 0..max_size {
            let i = new_instruction(mach);
            bb.push(i);
        }
        bb
    }

    fn spawn(
        &self,
        mach: Machine,
    ) -> BasicBlockSpawn {
        let parent: BasicBlock = BasicBlock {
            instructions: self.instructions.clone(),
        };
        BasicBlockSpawn {
            parent,
            mutant: self.clone(),
            ncount: 0,
            mach,
        }
    }

    fn len(&self) -> usize {
        self.instructions.len()
    }

    fn remove(&mut self, offset: usize) -> Instruction {
        self.instructions.remove(offset)
    }

    fn insert(&mut self, offset: usize, instr: Instruction) {
        self.instructions.insert(offset, instr)
    }

    fn push(&mut self, instr: Instruction) {
        self.instructions.push(instr)
    }
}

impl Index<usize> for BasicBlock {
    type Output = Instruction;

    fn index(&self, offset: usize) -> &Self::Output {
        &self.instructions[offset]
    }
}

impl IndexMut<usize> for BasicBlock {
    fn index_mut(&mut self, offset: usize) -> &mut Self::Output {
        &mut self.instructions[offset]
    }
}

fn run_program(prog: &BasicBlock, test_run: &TestRun, test: &Test) -> Option<State> {
    let mut s = State::new();

    for param in test_run.ins.iter().zip(test.ins.iter()) {
        s.set_i8(param.0.register, Some(*param.1));
    }
    if prog
        .instructions
        .iter()
        .all(|i| i.operate(&mut s))
    {
        Some(s)
    } else {
        None
    }
}

pub fn difference(prog: &BasicBlock, test_run: &TestRun) -> f64 {
    let mut ret: f64 = 0.0;
    for tc in test_run.tests.iter() {
        if let Some(state) = run_program(prog, test_run, tc) {
            for param in test_run.outs.iter().zip(tc.outs.iter()) {
                if let Some(v) = state.get_i8(param.0.register) {
                    let d: f64 = v.into();
                    let e: f64 = (*param.1).into();
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

fn cost(prog: &BasicBlock) -> f64 {
    /* quick and simple cost function,
     * number of instructions in the program.
     * Not really a bad thing to minimise for.
     */
    prog.len() as f64
}

fn mutate_delete(prog: &mut BasicBlock) {
    if prog.len() > 1 {
        let offset: usize = rand::thread_rng().gen_range(0, prog.len());
        prog.remove(offset);
    }
}

fn mutate_insert(
    prog: &mut BasicBlock,
    mach: Machine,
) {
    let offset: usize = if prog.len() > 0 {
        rand::thread_rng().gen_range(0, prog.len())
    } else {
        0
    };
    let instruction = new_instruction(mach);
    prog.insert(offset, instruction);
}

fn mutate(
    prog: &mut BasicBlock,
    mach: Machine,
) {
    let mutate: usize = rand::thread_rng().gen_range(0, 3);
    match mutate {
        /* randomize an instruction
         * (this could involve changing an operand, addressing mode, etc etc.
         */
        0 => {
            if prog.len() > 1 {
                let offset: usize = rand::thread_rng().gen_range(0, prog.len());
                prog[offset].randomize();
            }
        }
        /* delete an instruction */
        1 => {
            mutate_delete(prog);
        }
        /* insert a new instruction */
        2 => {
            mutate_insert(prog, mach);
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
    convergence: &dyn Fn(&BasicBlock) -> f64,
    prog: &BasicBlock,
) -> BasicBlock {
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

pub fn quick_dce(convergence: &dyn Fn(&BasicBlock) -> f64, prog: &BasicBlock) -> BasicBlock {
    let mut better = prog.clone();
    let score = convergence(prog);
    let mut cur: usize = 0;

    loop {
        let mut putative = better.clone();
        if cur >= better.len() {
            return better;
        }
        putative.remove(cur);
        if convergence(&putative) <= score {
            better = putative.clone();
        } else {
            cur += 1;
        }
    }
}

pub fn optimize(
    convergence: &dyn Fn(&BasicBlock) -> f64,
    prog: &BasicBlock,
    mach: Machine,
) -> BasicBlock {
    let mut population: Vec<(f64, BasicBlock)> = vec![];

    let fitness = convergence(prog);
    let ccost = cost(prog);
    population.push((cost(prog), prog.clone()));

    let best = prog;

    // if we find a better version, try to optimize that as well.
    if let Some(s) = best
        .spawn(mach)
        .take(1000000)
        .filter(|s| convergence(s) <= fitness)
        .map(|s| (cost(&s), s))
        .min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"))
    {
        if s.0 < ccost {
            return optimize(convergence, &s.1, mach);
        }
    }

    // Otherwise just return what we got.
    prog.clone()
}

pub fn stochastic_search(
    convergence: &dyn Fn(&BasicBlock) -> f64,
    mach: Machine,
) -> BasicBlock {
    // Initial population of a bajillion stupid programs
    // which are of course unlikely to be any good
    let mut population: Vec<(f64, BasicBlock)> = vec![];
    for _i in 1..1000 {
        let program = BasicBlock::initial_guess(mach, 20);
        population.push((convergence(&program), program));
    }

    loop {
        // Get the best specimen
        let b = population
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"))
            .unwrap();

        // get rid of unnecessary instructions
        let best = quick_dce(convergence, &b.1);

        if b.0 < 0.1 {
            return dead_code_elimination(convergence, &quick_dce(convergence, &b.1));
        }

        let mut next_generation: Vec<(f64, BasicBlock)> = vec![];

        for s in best
            .spawn(mach)
            .take(500)
        {
            let fit = convergence(&s);
            if fit < b.0 {
                let d = quick_dce(convergence, &s);
                next_generation.push((fit, d));
            }
        }

        if !next_generation.is_empty() {
            population = next_generation;
        }
    }
}
