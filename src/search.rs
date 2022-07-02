use crate::machine::Instruction;
use crate::machine::Machine;
use rand::{thread_rng, Rng};

trait Strop {
    fn random() -> Self;
    fn mutate(self) -> Self;
}

#[derive(Clone)]
pub struct BasicBlock<'a, State, Operand, OUD, IUD> {
    pub instructions: Vec<Instruction<'a, State, Operand, OUD, IUD>>,
}

impl<'a, State, Operand, OUD, IUD> Strop for BasicBlock<'_, State, Operand, OUD, IUD> {
    fn random() {
        // a new random basic block
        let mut bb = BasicBlock {
            instructions: vec![],
        };
        for _ in 0..20 {
            bb.push(Instruction::<'a, State, Operand, OUD, IUD>::random());
        }
    }

    fn mutate(&mut self) {
        randomly!(
        {
            /* randomize an instruction
             * (this could involve changing an operand, addressing mode, etc etc.
             */
            if !self.instructions.is_empty() {
                let offset = self.random_offset();
                self[offset].mutate();
            }
        }
        {
            /* delete an instruction */
            self.mutate_delete();
        }
        {
            /* insert a new instruction */
            self.mutate_insert();
        }
        {
            if self.instructions.len() > 2 {
                /* Pick two instructions and swap them round */
                let offset_a = self.random_offset();
                let offset_b = self.random_offset();
                let ins_a = self[offset_a];
                let ins_b = self[offset_b];
                self[offset_a] = ins_b;
                self[offset_b] = ins_a;
            }
        })
    }
}

impl<'a, State, Operand, OUD, IUD> BasicBlock<'_, State, Operand, OUD, IUD> {
    fn len(&self) -> usize {
        self.instructions.iter().map(|i| i.len()).sum()
    }

    fn remove(&mut self, offset: usize) -> Instruction<'_, State, Operand, OUD, IUD> {
        self.instructions.remove(offset)
    }

    fn insert(&mut self, offset: usize, instr: Instruction<'_, State, Operand, OUD, IUD>) {
        self.instructions.insert(offset, instr)
    }

    fn push(&mut self, instr: Instruction<'_, State, Operand, OUD, IUD>) {
        self.instructions.push(instr)
    }

    fn random_offset(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..self.instructions.len())
    }

    fn mutate_delete(&mut self) {
        let instr_count = self.instructions.len();
        if instr_count > 1 {
            self.remove(self.random_offset());
        }
    }

    fn mutate_insert(&mut self) {
        let instr_count = self.instructions.len();
        let offset: usize = if instr_count > 0 {
            self.random_offset()
        } else {
            0
        };
        self.insert(offset, Instruction::new());
    }
}

fn cost<State, Operand, OUD, IUD>(prog: &BasicBlock<'_, State, Operand, OUD, IUD>) -> f64 {
    /* quick and simple cost function,
     * number of instructions in the program.
     * Not really a bad thing to minimise for.
     */
    prog.len() as f64
}

pub fn quick_dce<'a, State, Operand, OUD, IUD>(
    correctness: &dyn Fn(&BasicBlock<'_, State, Operand, OUD, IUD>) -> f64,
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

pub fn stochastic_search<State, Operand, OUD, IUD>(
    cost: fn(Instruction<'_, State, Operand, OUD, IUD>) -> f64,
    mach: Machine<State, Operand, OUD, IUD>,
) -> BasicBlock<'_, State, Operand, OUD, IUD>
where
    IUD: Clone,
    OUD: Clone,
    Operand: Clone,
    State: Clone,
{
    let mut population: Vec<(f64, BasicBlock<'_, State, Operand, OUD, IUD>)> = vec![];
    let mut winners: Vec<BasicBlock<'_, State, Operand, OUD, IUD>> = vec![];
    let mut generation: u64 = 1;

    population.push(BasicBlock::new::<'_, State, Operand, OUD, IUD>());
    while winners.is_empty() {
        let best_score = population[0].0;

        // Spawn more specimens for next generation by mutating the current ones
        let population_size = if best_score < 500.0 { 10 } else { 50 };
        let mut ng: Vec<(f64, BasicBlock<'_, State, Operand, OUD, IUD>)> = population
            .iter()
            .flat_map(|s| {
                NextGeneration::new(mach, correctness, best_score, s.1.clone())
                    .collect::<Vec<(f64, BasicBlock<'_, State, Operand, OUD, IUD>)>>()
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
