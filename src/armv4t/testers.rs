//! Module containing testers for ARM. A tester in this context means a filter over a bruteforce
//! search, which filters only the candidate programs that correctly compute the given function.

use crate::armv4t::emulators::ArmV4T;
use crate::armv4t::instruction_set::Thumb;
use crate::armv4t::instruction_set::ThumbInstructionSet;
use crate::BruteForceSearch;
use crate::Candidate;
use crate::SearchFeedback;
use crate::InstructionSet;
use crate::Instruction;


/// Tests the candidate programs visited by a bruteforce search to see if they compute the given
/// function, taking two 32-bit integers and return one 32-bit integer, and also match the AAPCS32
/// calling convention.
#[derive(Debug)]
pub struct Aapcs32<S>
where S: SearchFeedback,
      S: Iterator<Item = Candidate<Thumb>>
{
    inputs: Vec<(i32, i32)>,
    search: S,
    func: fn(i32, i32) -> Option<i32>,
}

impl<S: Iterator<Item = Candidate<Thumb>> + SearchFeedback> Aapcs32<S> {
    /// Returns a new Aapcs32 struct
    pub fn new(
        search: S,
        func: fn(i32, i32) -> Option<i32>,
    ) -> Self {
        Self {
            inputs: vec![],
            search,
            func,
        }
    }

    fn test1(&self, candidate: &<S as Iterator>::Item, a: i32, b: i32) -> u32 {
        use crate::Emulator;
        let mut emu = ArmV4T::default();

        if let Some(result) = (self.func)(a, b) {
            emu.set_r0(a);
            emu.set_r1(b);
            emu.run(0x8000, candidate);
            (emu.get_r0() ^ result).count_ones()
        } else {
            0
        }
    }

    fn test(&mut self, candidate: &Candidate<Thumb>) -> u32 {
        use rand::random;
        use rand::Rng;

        let mut score = 0;

        // Try the values that have returned false before
        for inputs in &self.inputs {
            score += self.test1(candidate, inputs.0, inputs.1);
        }

        // Try ten more random value pairs across a small range to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a: i32 = rand::thread_rng().gen_range(-100..100);
            let b: i32 = rand::thread_rng().gen_range(-100..100);
            let score1 = self.test1(candidate, a, b);
            if score1 != 0 {
                self.inputs.push((a, b));
                score += score1;
            }
        }

        // Try ten more random value pairs to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a: i32 = random();
            let b: i32 = random();
            let score1 = self.test1(candidate, a, b);
            if score1 != 0 {
                self.inputs.push((a, b));
                score += score1;
            }
        }
        score
    }
}

impl<S: Iterator<Item = Candidate<Thumb>> + SearchFeedback> Iterator for Aapcs32<S> 
{
    type Item = Candidate<Thumb>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.search.next() {
            if self.test(&candidate) == 0 {
                return Some(candidate);
            }
        }
        None
    }
}
