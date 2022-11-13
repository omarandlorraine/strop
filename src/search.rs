//! The `search` module provides conveniences for searching for the target
//! sequence.

use crate::machine::Instruction;
use crate::randomly;
use rand::{thread_rng, Rng};
use std::ops::{Index, IndexMut};

/// A [basic block](https://en.wikipedia.org/wiki/Basic_block) is a sequence of
/// instructions that contains no jumps or branches. Another key word for this
/// is "branchless" or "branch-free". This property guarantees that the sequence
/// of instructions does not loop or halt, and makes it amenable to certain
/// kinds of optimizations.
#[derive(Clone, Debug)]
pub struct BasicBlock<I>
where
    I: Instruction,
{
    /// The list of instructions in the basic block
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
    fn mutation(&self) -> BasicBlock<I> {
        let mut r = self.clone();
        mutate(&mut r);
        r
    }

    pub fn is_empty(&self) -> bool {
        //! Returns true if this basic block contains no instructions, or false if it contains at least
        //! one instruction.
        self.instructions.is_empty()
    }

    pub fn len(&self) -> usize {
        //! returns the length of the program, in machine words
        self.instructions.iter().map(|i| i.length()).sum()
    }

    fn remove(&mut self, offset: usize) -> I {
        self.instructions.remove(offset)
    }

    fn insert(&mut self, offset: usize, instr: I) {
        self.instructions.insert(offset, instr)
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
    let instruction = I::new();
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

/// Search for a basic block. Supply this function with a cost function; `stochastic_search` will
/// halt when the cost function returns zero.
pub fn stochastic_search<I: Instruction + Clone, F>(cost: F) -> BasicBlock<I>
where
    F: Fn(&BasicBlock<I>) -> f64,
{
    let mut population: Vec<(f64, BasicBlock<I>)> = vec![];
    let mut winners: Vec<BasicBlock<I>> = vec![];

    let default = BasicBlock::<I>::default();
    population.push((cost(&default), default));

    while winners.is_empty() {
        let best_score = population[0].0;

        // Spawn more specimens for next generation by mutating the current ones
        let population_size = if best_score < 500.0 { 10 } else { 50 };
        let mut ng: Vec<(f64, BasicBlock<I>)> = population
            .iter()
            .map(|s| s.1.mutation())
            .map(|s| (cost(&s), s))
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
            let _nbest = population[0].0;

            population.truncate(50);
        }
    }

    winners[0].clone()
}
