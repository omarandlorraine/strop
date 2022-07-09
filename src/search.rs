use crate::machine::Instruction;
use crate::machine::Machine;
use rand::{thread_rng, Rng};

pub trait Strop {
    fn random() -> Self;
    fn mutate(&mut self);
}

#[derive(Clone)]
pub struct BasicBlock<'a, State, Operand, OUD, IUD> {
    pub instructions: Vec<Instruction<'a, State, Operand, OUD, IUD>>,
}

impl<'a, State, Operand, OUD, IUD> Strop for BasicBlock<'_, State, Operand, OUD, IUD> {
    fn random() -> Self {
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
                self.instructions[offset].mutate();
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
                let ins_a = self.instructions[offset_a];
                let ins_b = self.instructions[offset_b];
                self.instructions[offset_a] = ins_b;
                self.instructions[offset_b] = ins_a;
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
        self.insert(offset, Instruction::<'a, State, Operand, OUD, IUD>::new());
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

pub fn stochastic_step<'r, State, Operand, OUD, IUD>(
    cost_fn: fn(BasicBlock<'_, State, Operand, OUD, IUD>) -> f64,
    specimen: (f64, BasicBlock<'r, State, Operand, OUD, IUD>),
) -> (f64, BasicBlock<'r, State, Operand, OUD, IUD>)
where
    IUD: Clone,
    OUD: Clone,
    Operand: Clone,
    State: Clone,
{
    let mut next = specimen.1.clone();
    next.mutate();
    let next_cost = cost_fn(next);
    if next_cost < specimen.0 {
        (next_cost, next)
    } else {
        specimen
    }
}

pub fn stochastic_search<'r, State, Operand, OUD, IUD>(
    cost: fn(BasicBlock<'_, State, Operand, OUD, IUD>) -> f64,
    mach: Machine,
) -> BasicBlock<'r, State, Operand, OUD, IUD>
where
    IUD: Clone,
    OUD: Clone,
    Operand: Clone,
    State: Clone,
{
    // function to create a completely random basic block and its cost
    fn newbie<'r, State, Operand, OUD, IUD>(
        cost: fn(BasicBlock<'_, State, Operand, OUD, IUD>) -> f64,
    ) -> (f64, BasicBlock<'r, State, Operand, OUD, IUD>) {
        let n = BasicBlock::<'r, State, Operand, OUD, IUD>::random();
        (cost(n), n)
    }

    // get the best speciment in the population
    fn best<'r, State, Operand, OUD, IUD>(
        population: Vec<(f64, BasicBlock<'r, State, Operand, OUD, IUD>)>,
    ) -> (f64, BasicBlock<'r, State, Operand, OUD, IUD>) {
        *population
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"))
            .unwrap()
    }

    // an initial population of 10 randomers
    let mut population: Vec<(f64, BasicBlock<'r, State, Operand, OUD, IUD>)> =
        (0..10).map(|_| newbie(cost)).collect();
    let mut best_cost = best(population).0;

    while best_cost > 0.1 {
        population = population
            .iter()
            .map(|&s| stochastic_step(cost, s))
            .collect();
        best_cost = best(population).0;
    }

    best(population).1
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
