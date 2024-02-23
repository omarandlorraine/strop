//! Module containing testers for ARM. A tester in this context means a filter over a bruteforce
//! search, which filters only the candidate programs that correctly compute the given function.
use crate::Fitness;

use crate::armv4t::emulators::ArmV4T;
use crate::armv4t::instruction_set::Thumb;

use crate::Candidate;
use crate::SearchAlgorithm;

/// Tests the candidate programs visited by a search stategy to see if they compute the given
/// function, taking two 32-bit integers and return one 32-bit integer, and also match the AAPCS32
/// calling convention.
#[derive(Debug)]
pub struct Aapcs32<S>
where
    S: SearchAlgorithm<Item = Thumb>,
{
    inputs: Vec<(i32, i32)>,
    search: S,
    func: fn(i32, i32) -> Option<i32>,
}

impl<S: SearchAlgorithm<Item = Thumb>> Aapcs32<S> {
    /// Returns a new Aapcs32 struct
    pub fn new(search: S, func: fn(i32, i32) -> Option<i32>) -> Self {
        use rand::random;
        use rand::Rng;
        let mut inputs: Vec<(i32, i32)> = vec![];
        for _ in 0..100 {
            let a: i32 = rand::thread_rng().gen_range(-100..100);
            let b: i32 = rand::thread_rng().gen_range(-100..100);
            if func(a, b).is_some() {
                inputs.push((a, b));
            }
            let a: i32 = random();
            let b: i32 = random();
            if func(a, b).is_some() {
                inputs.push((a, b));
            }
        }
        Self {
            inputs,
            search,
            func,
        }
    }

    fn possible_test_case(&mut self, candidate: &Candidate<Thumb>, a: i32, b: i32) -> Option<i32> {
        use crate::Emulator;
        if let Some(result) = (self.func)(a, b) {
            let mut emu = ArmV4T::default();
            emu.set_r0(a);
            emu.set_r1(b);
            emu.run(0x8000, candidate);
            if emu.get_r0() != result {
                self.inputs.push((a, b));
                return Some(result);
            }
        }
        None
    }

    fn test1(&self, candidate: &Candidate<Thumb>, a: i32, b: i32) -> u32 {
        use crate::Emulator;
        if let Some(result) = (self.func)(a, b) {
            let mut emu = ArmV4T::default();
            emu.set_r0(a);
            emu.set_r1(b);
            emu.run(0x8000, candidate);
            (emu.get_r0() ^ result).count_ones()
        } else {
            0
        }
    }

    fn correctness(&self, candidate: &Candidate<Thumb>) -> u32 {
        let mut score = 0;
        // Try the values that have returned false before
        for inputs in &self.inputs {
            score += self.test1(candidate, inputs.0, inputs.1);
        }
        score
    }

    fn test(&mut self, candidate: &Candidate<Thumb>) -> u32 {
        use rand::random;
        use rand::Rng;

        let mut score = self.correctness(candidate);
        if score > 0 {
            return score;
        }

        // Try ten more random value pairs across a small range to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a: i32 = rand::thread_rng().gen_range(-100..100);
            let b: i32 = rand::thread_rng().gen_range(-100..100);
            self.possible_test_case(candidate, a, b);
            score += self.test1(candidate, a, b);

            let a: i32 = random();
            let b: i32 = random();
            self.possible_test_case(candidate, a, b);
            score += self.test1(candidate, a, b);
        }
        score
    }
}

impl<S: SearchAlgorithm<Item = Thumb>> SearchAlgorithm for Aapcs32<S> {
    type Item = Thumb;

    fn fitness(&mut self, candidate: &Candidate<Thumb>) -> Fitness {
        match self.search.fitness(candidate) {
            Fitness::FailsStaticAnalysis => Fitness::FailsStaticAnalysis,
            Fitness::Passes(_) => Fitness::Passes(self.test(candidate) as f32),
        }
    }

    fn score(&mut self, score: f32) {
        self.search.score(score);
    }

    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>) {
        self.search.replace(offset, instruction);
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        while let Some(candidate) = self.search.generate() {
            let score = self.test(&candidate);
            self.search.score(score as f32);
            if score == 0 {
                // We've found a program that passes the test cases we've found; let's optimize the
                // program.
                return Some(candidate);
            }
        }
        None
    }
}
