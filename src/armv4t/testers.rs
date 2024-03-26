//! Module containing testers for ARM. A tester in this context means a filter over a bruteforce
//! search, which filters only the candidate programs that correctly compute the given function.
use crate::Fitness;

use crate::armv4t::emulators::ArmV4T;
use crate::armv4t::instruction_set::Thumb;

use crate::Candidate;
use crate::Scalar;
use crate::SearchAlgorithm;

/// Tests the candidate programs visited by a search strategy to see if they compute the given
/// function, taking two 32-bit integers and return one 32-bit integer, and also match the AAPCS32
/// calling convention.
#[derive(Debug)]
pub struct Aapcs32<S, T, U, V>
where
    S: SearchAlgorithm<Item = Thumb>,
    T: Scalar,
    U: Scalar,
    V: Scalar,
{
    inputs: Vec<(T, U)>,
    search: S,
    func: fn(T, U) -> Option<V>,
}

impl<S, T, U, V> Aapcs32<S, T, U, V>
where
    S: SearchAlgorithm<Item = Thumb> + Sized,
    T: Scalar,
    U: Scalar,
    V: Scalar,
{
    /// Returns a new Aapcs32 struct
    pub fn new(search: S, func: fn(T, U) -> Option<V>) -> Self {
        let mut inputs: Vec<(T, U)> = vec![];
        for _ in 0..100 {
            let a = T::random();
            let b = U::random();
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

    fn possible_test_case(&mut self, candidate: &Candidate<Thumb>, a: T, b: U) -> Option<V> {
        use crate::Emulator;
        if let Some(result) = (self.func)(a, b) {
            let mut emu = ArmV4T::default();
            emu.set_r0(a.as_i32());
            emu.set_r1(b.as_i32());
            emu.run(0x8000, candidate);
            if emu.get_r0() != result.as_i32() {
                self.inputs.push((a, b));
                return Some(result);
            }
        }
        None
    }

    fn test1(&self, candidate: &Candidate<Thumb>, a: T, b: U) -> u32 {
        use crate::Emulator;
        if let Some(result) = (self.func)(a, b) {
            let mut emu = ArmV4T::default();
            emu.set_r0(a.as_i32());
            emu.set_r1(b.as_i32());
            emu.run(0x8000, candidate);
            result.hamming(emu.get_r0())
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
        let mut score = self.correctness(candidate);
        if score > 0 {
            return score;
        }

        // Try ten more random value pairs across a small range to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a = T::random();
            let b = U::random();
            self.possible_test_case(candidate, a, b);
            score += self.test1(candidate, a, b);
        }
        score
    }
}

impl<S: SearchAlgorithm<Item = Thumb>, T: Scalar, U: Scalar, V: Scalar> SearchAlgorithm
    for Aapcs32<S, T, U, V>
{
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
