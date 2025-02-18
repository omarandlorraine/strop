//! Implements the `Mutate trait for `Sequence`

use crate::Mutate;
use crate::Sequence;
use rand::Rng;

impl<Insn: Mutate> Sequence<Insn> {
    fn random_offset(&self) -> usize {
        let mut rng = rand::thread_rng();
        if self.0.is_empty() {
            0
        } else {
            rng.gen_range(0..self.0.len())
        }
    }

    /// Deletes a randomly selected instruction
    fn delete_random(&mut self) {
        if !self.is_empty() {
            let offset = self.random_offset();
            self.0.remove(offset);
        }
    }

    /// Insert a randomly generated instruction at a random location in the program.
    fn insert_random(&mut self) {
        let offs = self.random_offset();
        self.0.insert(offs, Insn::random());
    }

    /// If the length of instructions is two or more, then pick two instructions at random and swap
    /// them over.
    fn exchange_random(&mut self) {
        if self.len() > 2 {
            let offset_a = self.random_offset();
            let offset_b = self.random_offset();
            self.0.swap(offset_a, offset_b);
        }
    }

    /// If the `Sequence` contains at least one instruction, then replace one instruction with
    /// something totally different
    fn replace_random(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and swap it for something totally different.
        if !self.is_empty() {
            let offset = self.random_offset();
            self.0[offset] = Insn::random();
        }
    }

    /// Randomly mutate one of the instructions in the `Sequence`
    fn mutate_random(&mut self) {
        // If the list of instructions contains at least one instruction, then pick one at random
        // and call its `mutate` method.
        if !self.0.is_empty() {
            let offset = self.random_offset();
            self.0[offset].mutate();
        }
    }
}

impl<Insn: Mutate> Mutate for Sequence<Insn> {
    fn random() -> Self {
        let mut s = Self(vec![]);
        while rand::random() {
            s.insert_random()
        }
        s
    }

    fn mutate(&mut self) {
        use rand::Rng;
        let choice = rand::thread_rng().gen_range(0..5);

        match choice {
            0 => self.delete_random(),
            1 => self.insert_random(),
            2 => self.exchange_random(),
            3 => self.replace_random(),
            4 => self.mutate_random(),
            _ => unreachable!(),
        }
    }
}

impl<Insn: Copy> crate::Crossover for Sequence<Insn> {
    fn crossover(a: &Self, b: &Self) -> Self {
        let min_len = usize::min(a.len(), b.len());
        let crossover_point = if min_len == 0 {
            0
        } else {
            rand::thread_rng().gen_range(0..min_len)
        };
        Self(
            a.iter()
                .take(crossover_point)
                .chain(b.iter().skip(crossover_point))
                .copied()
                .collect(),
        )
    }
}
