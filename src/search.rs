use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Width;
use crate::{State, Step, TestRun};
use rand::{thread_rng, Rng};
use std::ops::{Index, IndexMut};
use strop::randomly;

#[derive(Clone)]
pub struct BasicBlock<I>
where
    I: Instruction,
{
    pub instructions: Vec<I>,
}

impl<I: Instruction> Default for BasicBlock<I> {
    fn default() -> Self {
        BasicBlock {
            instructions: vec![],
        }
    }
}

struct BasicBlockSpawn<I: Instruction> {
    parent: BasicBlock<I>,
    mutant: BasicBlock<I>,
    ncount: usize,
    mach: Machine,
}

impl<I: Instruction> Iterator for BasicBlockSpawn<I> {
    type Item = BasicBlock<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ncount == 0 {
            self.mutant = self.parent.clone();
            self.ncount = 100;
        }
        self.ncount -= 1;
        mutate(&mut self.mutant);
        Some(self.mutant.clone())
    }
}

impl<I: Instruction + Clone> BasicBlock<I> {
    fn initial_guess(mach: Machine, max_size: i32) -> BasicBlock<I> {
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        for _i in 0..max_size {
            let i = I::random();
            bb.push(i);
        }
        bb
    }

    fn spawn(&self, mach: Machine) -> BasicBlockSpawn<I> {
        let parent: BasicBlock<I> = BasicBlock {
            instructions: self.instructions.clone(),
        };
        BasicBlockSpawn {
            parent,
            mutant: self.clone(),
            ncount: 0,
            mach,
        }
    }

    fn mutation(&self) -> BasicBlock<I> {
        let mut r = self.clone();
        mutate(&mut r);
        r
    }

    fn len(&self) -> usize {
        self.instructions.iter().map(|i| i.len()).sum()
    }

    fn remove(&mut self, offset: usize) -> I {
        self.instructions.remove(offset)
    }

    fn insert(&mut self, offset: usize, instr: I) {
        self.instructions.insert(offset, instr)
    }

    fn push(&mut self, instr: I) {
        self.instructions.push(instr)
    }

    fn random_offset(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..self.instructions.len())
    }
}

impl<I: Instruction> Index<usize> for BasicBlock<I> {
    type Output = I;

    fn index(&self, offset: usize) -> &Self::Output {
        &self.instructions[offset]
    }
}

impl<I: Instruction> IndexMut<usize> for BasicBlock<I> {
    fn index_mut(&mut self, offset: usize) -> &mut Self::Output {
        &mut self.instructions[offset]
    }
}

pub fn difference<I: Instruction>(prog: &BasicBlock<I>, test_run: &TestRun) -> f64 {
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

fn cost<I: Instruction>(prog: &BasicBlock<I>) -> f64 {
    /* quick and simple cost function,
     * number of instructions in the program.
     * Not really a bad thing to minimise for.
     */
    prog.len() as f64
}

fn mutate_delete<I: Instruction>(prog: &mut BasicBlock<I>) {
    let instr_count = prog.instructions.len();
    if instr_count > 1 {
        prog.remove(prog.random_offset());
    }
}

fn mutate_insert<I: Instruction>(prog: &mut BasicBlock<I>) {
    let instr_count = prog.instructions.len();
    let offset: usize = if instr_count > 0 {
        prog.random_offset()
    } else {
        0
    };
    let instruction = I::random();
    prog.insert(offset, instruction);
}

fn mutate<I: Instruction>(prog: &mut BasicBlock<I>) {
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
        mutate_insert(prog);
    }
    )
}

pub fn quick_dce<I: Instruction>(
    correctness: &dyn Fn(&BasicBlock<I>) -> f64,
    prog: &BasicBlock<I>,
) -> BasicBlock<I> {
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

pub fn optimize<I: Instruction>(
    correctness: &TestRun,
    prog: &BasicBlock<I>,
    mach: Machine,
) -> BasicBlock<I> {
    let mut population: Vec<(f64, BasicBlock<I>)> = vec![];

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

pub fn stochastic_search<I: Instruction + Clone>(
    correctness: &TestRun,
    mach: Machine,
) -> BasicBlock<I> {
    let mut population: Vec<(f64, BasicBlock<I>)> = vec![];
    let mut winners: Vec<BasicBlock<I>> = vec![];
    let mut generation: u64 = 1;

    let default = BasicBlock::<I>::default();
    population.push((difference(&default, correctness), default));

    while winners.is_empty() {
        let best_score = population[0].0;

        // Spawn more specimens for next generation by mutating the current ones
        let population_size = if best_score < 500.0 { 10 } else { 50 };
        let mut ng: Vec<(f64, BasicBlock<I>)> = population
            .iter()
            .map(|s| s.1.mutation())
            .map(|s| (difference(&s, correctness), s))
            .collect();

        // concatenate the current generation to the next
        for s in population.clone().into_iter().take(population_size) {
            if s.0 < 0.1 {
                winners.push(s.1);
            } else {
                ng.push(s);
            }
        }

        if !ng.is_empty() {
            // Sort the population by score.
            ng.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"));

            population = ng;
            let nbest = population[0].0;

            population.truncate(50);
            generation += 1;
        }
    }

    winners[0].clone()
}
