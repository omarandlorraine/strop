use crate::disassemble;
use crate::machine::new_instruction;
use crate::machine::Instruction;
use crate::machine::Width;
use crate::{Machine, State, Step, TestRun};
use rand::{thread_rng, Rng};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelSliceMut;
use std::ops::{Index, IndexMut};
use strop::randomly;

#[derive(Clone)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

struct BasicBlockSpawn {
    parent: BasicBlock,
    mutant: BasicBlock,
    ncount: usize,
    mach: Machine,
}

impl Iterator for BasicBlockSpawn {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ncount == 0 {
            self.mutant = self.parent.clone();
            self.ncount = 100;
        }
        self.ncount -= 1;
        mutate(&mut self.mutant, self.mach);
        Some(self.mutant.clone())
    }
}

impl BasicBlock {
    fn initial_guess(mach: Machine, max_size: i32) -> BasicBlock {
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        for _i in 0..max_size {
            let i = new_instruction(mach);
            bb.push(i);
        }
        bb
    }

    fn spawn(&self, mach: Machine) -> BasicBlockSpawn {
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
        self.instructions.iter().map(|i| i.len()).sum()
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

    fn random_offset(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..self.instructions.len())
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

pub fn difference(prog: &BasicBlock, test_run: &TestRun) -> f64 {
    let mut ret: f64 = 0.0;

    for tc in test_run.tests.iter() {
        let mut s = State::new();

        for step in &tc.steps {
            match step {
                Step::Run => {
                    let mut pc: usize = 0;
                    for _i in 0..100 {
                        if pc == prog.instructions.len() {
                            break;
                        }
                        if let Some(new_pc) = prog.instructions[pc].operate(&mut s).newpc(pc) {
                            pc = new_pc;
                            if pc > prog.instructions.len() {
                                ret += 1000.0;
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                Step::Set(datum, val) => match datum.width() {
                    Width::Width8 => {
                        s.set_i8(*datum, Some(*val as i8));
                    }
                    Width::Width16 => {
                        s.set_i16(*datum, Some(*val as i16));
                    }
                },
                Step::Diff(datum, val) => {
                    if let Some(v) = s.get_i16(*datum) {
                        let d: f64 = (val - v as i32).into();
                        ret += d.abs();
                    } else {
                        ret += 65536.0;
                    }
                }
                Step::NonZero(datum) => {
                    if let Some(v) = s.get_i16(*datum) {
                        if v != 0 {
                            ret += 2.0;
                        }
                    } else {
                        ret += 100.0;
                    }
                }
                Step::Positive(datum) => {
                    if let Some(v) = s.get_i16(*datum) {
                        if v <= 0 {
                            ret += 2.0;
                        }
                    } else {
                        ret += 100.0;
                    }
                }
                Step::Negative(datum) => {
                    if let Some(v) = s.get_i16(*datum) {
                        if v >= 0 {
                            ret += 2.0;
                        }
                    } else {
                        ret += 100.0;
                    }
                }
                Step::Ham(datum, val, dontcare) => {
                    if let Some(v) = s.get_i16(*datum) {
                        ret += (((v as i32) ^ val) & dontcare).count_ones() as f64;
                    } else {
                        ret += 1000.0
                    }
                }
            }
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
    let instr_count = prog.instructions.len();
    if instr_count > 1 {
        prog.remove(prog.random_offset());
    }
}

fn mutate_insert(prog: &mut BasicBlock, mach: Machine) {
    let instr_count = prog.instructions.len();
    let offset: usize = if instr_count > 0 {
        prog.random_offset()
    } else {
        0
    };
    let instruction = new_instruction(mach);
    prog.insert(offset, instruction);
}

fn mutate(prog: &mut BasicBlock, mach: Machine) {
    randomly!(
    {
        /* randomize an instruction
         * (this could involve changing an operand, addressing mode, etc etc.
         */
        if !prog.instructions.is_empty() {
            let offset = prog.random_offset();
            prog[offset].randomize();
        }
    }
    {
        /* delete an instruction */
        mutate_delete(prog);
    }
    {
        /* insert a new instruction */
        mutate_insert(prog, mach);
    }
    {
        if prog.instructions.len() > 2 {
            /* Pick two instructions and swap them round */
            let offset_a = prog.random_offset();
            let offset_b = prog.random_offset();
            let ins_a = prog[offset_a];
            let ins_b = prog[offset_b];
            prog[offset_a] = ins_b;
            prog[offset_b] = ins_a;
        }
    })
}

pub struct InitialPopulation<'a> {
    mach: Machine,
    testrun: &'a TestRun,
}

impl<'a> InitialPopulation<'a> {
    fn new(mach: Machine, testrun: &TestRun) -> InitialPopulation {
        InitialPopulation { testrun, mach }
    }
}

impl Iterator for InitialPopulation<'_> {
    type Item = (f64, BasicBlock);

    fn next(&mut self) -> Option<Self::Item> {
        // Just spawn a random BasicBlock.
        let program = BasicBlock::initial_guess(self.mach, 20);

        // TODO: Should this check that the dce doesn't just empty the BB?
        let d = quick_dce(
            &|prog: &BasicBlock| difference(prog, self.testrun),
            &program,
        );
        Some((difference(&d, self.testrun), d))
    }
}

pub struct NextGeneration<'a> {
    testrun: &'a TestRun,
    bb: std::iter::Take<BasicBlockSpawn>,
    score: f64,
}

impl<'a> NextGeneration<'a> {
    fn new(mach: Machine, testrun: &'a TestRun, score: f64, bb: BasicBlock) -> NextGeneration {
        NextGeneration {
            testrun,
            bb: bb.spawn(mach).take(500),
            score,
        }
    }
}

impl<'a> Iterator for NextGeneration<'a> {
    type Item = (f64, BasicBlock);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(s) = self.bb.next() {
            let fit = difference(&s, self.testrun);
            if fit < self.score {
                let t = quick_dce(&|prog: &BasicBlock| difference(prog, self.testrun), &s);
                return Some((fit, t));
            }
        }
        None
    }
}

pub fn dead_code_elimination(
    correctness: &dyn Fn(&BasicBlock) -> f64,
    prog: &BasicBlock,
) -> BasicBlock {
    let mut better = prog.clone();

    for _m in 1..100 {
        let mut putative = prog.clone();
        for _n in 1..10 {
            mutate_delete(&mut putative);
            if correctness(&better) >= correctness(&putative) {
                better = putative.clone();
            } else {
                break;
            }
        }
    }
    better
}

pub fn quick_dce(correctness: &dyn Fn(&BasicBlock) -> f64, prog: &BasicBlock) -> BasicBlock {
    let mut better = prog.clone();
    let score = correctness(prog);
    let mut cur: usize = 0;

    loop {
        let mut putative = better.clone();
        if cur >= better.instructions.len() {
            return better;
        }
        putative.remove(cur);
        if correctness(&putative) <= score {
            better = putative.clone();
        } else {
            cur += 1;
        }
    }
}

pub fn optimize(correctness: &TestRun, prog: &BasicBlock, mach: Machine) -> BasicBlock {
    let mut population: Vec<(f64, BasicBlock)> = vec![];

    let fitness = difference(prog, correctness);
    let ccost = cost(prog);
    population.push((cost(prog), prog.clone()));

    let best = prog;

    // if we find a better version, try to optimize that as well.
    if let Some(s) = best
        .spawn(mach)
        .take(1000)
        .filter(|s| difference(s, correctness) <= fitness)
        .map(|s| (cost(&s), s))
        .min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"))
    {
        if s.0 < ccost {
            return optimize(correctness, &s.1, mach);
        }
    }

    // Otherwise just return what we got.
    prog.clone()
}

pub fn stochastic_search(
    correctness: &TestRun,
    mach: Machine,
    graph: bool,
    debug: bool,
) -> BasicBlock {
    let mut init = InitialPopulation::new(mach, correctness);

    let mut population: Vec<(f64, BasicBlock)> = vec![];
    let mut winners: Vec<BasicBlock> = vec![];
    let mut generation: u64 = 1;

    population.push(init.next().unwrap());

    while winners.is_empty() {
        let best_score = population[0].0;

        // Spawn more specimens for next generation by mutating the current ones
        let population_size = if best_score < 500.0 { 10 } else { 50 };
        let mut ng: Vec<(f64, BasicBlock)> = population
            .par_iter()
            .flat_map(|s| {
                NextGeneration::new(mach, correctness, best_score, s.1.clone())
                    .collect::<Vec<(f64, BasicBlock)>>()
            })
            .collect();

        // concatenate the current generation to the next
        for s in population.clone().into_iter().take(population_size) {
            if s.0 < 0.1 {
                winners.push(s.1);
            } else {
                ng.push(s);
            }
        }

        // Sort the population by score.
        ng.par_sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"));

        population = ng;
        let nbest = population[0].0;

        if graph {
            println!("{}, {}, {}", generation, population.len(), nbest);
        }
        if debug {
            disassemble(population[0].1.clone());
        }
        population.truncate(50);
        generation += 1;
    }

    winners[0].clone()
}

#[cfg(test)]
mod tests {
    use crate::search::mutate_delete;
    use crate::BasicBlock;
    use crate::Machine;

    #[test]
    fn delete_from_an_empty_bb() {
        let mut bb = BasicBlock::initial_guess(Machine::Stm8, 20);
        for _i in 0..500 {
            mutate_delete(&mut bb);
        }
    }
}
