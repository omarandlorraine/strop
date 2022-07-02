use crate::disassemble;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Width;
use crate::{Step, TestRun};
use rand::{thread_rng, Rng};
use strop::randomly;

trait Strop {
    fn random() -> Self;
    fn mutate(self) -> Self;
}

#[derive(Clone)]
pub struct BasicBlock<'a, State, Operand, OUD, IUD> {
    pub instructions: Vec<Instruction<'a, State, Operand, OUD, IUD>>,
}

impl<'a, State, Operand, OUD, IUD> BasicBlock<'_, State, Operand, OUD, IUD> {
    fn initial_guess(mach: Machine, max_size: i32) -> BasicBlock<'a, State, Operand, OUD, IUD> {
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        for _i in 0..max_size {
            let i = mach.new_instruction();
            bb.push(i);
        }
        bb
    }

    fn spawn(&self, mach: Machine) -> BasicBlockSpawn {
        let parent: BasicBlock<State, Operand, OUD, IUD> =
            BasicBlock::<'a, State, Operand, OUD, IUD> {
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

    fn remove(&mut self, offset: usize) -> Instruction<State, Operand, OUD, IUD> {
        self.instructions.remove(offset)
    }

    fn insert(&mut self, offset: usize, instr: Instruction<State, Operand, OUD, IUD>) {
        self.instructions.insert(offset, instr)
    }

    fn push(&mut self, instr: Instruction<State, Operand, OUD, IUD>) {
        self.instructions.push(instr)
    }

    fn random_offset(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..self.instructions.len())
    }

    fn mutate_delete(&mut self) {
        let instr_count = prog.instructions.len();
        if instr_count > 1 {
            prog.remove(prog.random_offset());
        }
    }

    fn mutate_insert(&mut self) {
        let instr_count = prog.instructions.len();
        let offset: usize = if instr_count > 0 {
            prog.random_offset()
        } else {
            0
        };
        let instruction = mach.new_instruction();
        prog.insert(offset, instruction);
    }

    fn mutate(&mut self) {
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
}

pub fn difference<State, Operand, OUD, IUD>(
    prog: &BasicBlock<State, Operand, OUD, IUD>,
    test_run: &TestRun,
) -> f64 {
    let mut ret: f64 = 0.0;

    for tc in test_run.tests.iter() {
        let mut s = State::new();

        for step in &tc.steps {
            match step {
                Step::Run => {
                    for i in &prog.instructions {
                        i.operate(&mut s);
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

fn cost<State, Operand, OUD, IUD>(prog: &BasicBlock<State, Operand, OUD, IUD>) -> f64 {
    /* quick and simple cost function,
     * number of instructions in the program.
     * Not really a bad thing to minimise for.
     */
    prog.len() as f64
}

pub fn quick_dce<'a, State, Operand, OUD, IUD>(
    correctness: &dyn Fn(&BasicBlock<State, Operand, OUD, IUD>) -> f64,
    prog: &BasicBlock<'a, State, Operand, OUD, IUD>,
) -> BasicBlock<'a, State, Operand, OUD, IUD> {
    let mut better = prog.clone();
    let score = correctness(prog);
    let mut cur: usize = 0;

    loop {
        let mut putative = better.clone();
        if cur >= better.instructions.len() {
            return *better;
        }
        putative.remove(cur);
        if correctness(&putative) <= score {
            better = putative.clone();
        } else {
            cur += 1;
        }
    }
}

pub fn optimize<'a, State, Operand, OUD, IUD>(
    correctness: &TestRun,
    prog: &BasicBlock<State, Operand, OUD, IUD>,
    mach: Machine,
) -> &'a BasicBlock<'a, State, Operand, OUD, IUD> {
    let mut population: Vec<(f64, BasicBlock<State, Operand, OUD, IUD>)> = vec![];

    let fitness = difference(prog, correctness);
    let ccost = cost(prog);
    population.push((cost(prog), *prog.clone()));

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

pub fn stochastic_search<State, Operand, OUD, IUD>(
    correctness: &TestRun,
    mach: Machine,
    graph: bool,
    debug: bool,
) -> BasicBlock<State, Operand, OUD, IUD>
where
    IUD: Clone,
    OUD: Clone,
    Operand: Clone,
    State: Clone,
{
    let mut init = InitialPopulation::new(mach, correctness);

    let mut population: Vec<(f64, BasicBlock<State, Operand, OUD, IUD>)> = vec![];
    let mut winners: Vec<BasicBlock<State, Operand, OUD, IUD>> = vec![];
    let mut generation: u64 = 1;

    population.push(init.next().unwrap());

    while winners.is_empty() {
        let best_score = population[0].0;

        // Spawn more specimens for next generation by mutating the current ones
        let population_size = if best_score < 500.0 { 10 } else { 50 };
        let mut ng: Vec<(f64, BasicBlock<State, Operand, OUD, IUD>)> = population
            .iter()
            .flat_map(|s| {
                NextGeneration::new(mach, correctness, best_score, s.1.clone())
                    .collect::<Vec<(f64, BasicBlock<State, Operand, OUD, IUD>)>>()
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

        if !ng.is_empty() {
            // Sort the population by score.
            ng.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"));

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
    }

    winners[0].clone()
}

#[cfg(test)]
mod tests {
    use crate::search::mutate_delete;
    use crate::BasicBlock;
    #[test]
    fn delete_from_basic_block() {
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        mutate_delete(&mut bb);
    }
}
