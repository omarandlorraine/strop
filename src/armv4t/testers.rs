//! Module containing testers for ARM. A tester in this context means a filter over a bruteforce
//! search, which filters only the candidate programs that correctly compute the given function.

use crate::armv4t::emulators::ArmV4T;
use crate::armv4t::instruction_set::Thumb;
use crate::armv4t::instruction_set::ThumbInstructionSet;
use crate::BruteForceSearch;
use crate::Candidate;

/// Tests the candidate programs visited by a bruteforce search to see if they compute the given
/// function, taking two 32-bit integers and return one 32-bit integer, and also match the AAPCS32
/// calling convention.
#[derive(Debug)]
pub struct Aapcs32 {
    inputs: Vec<(i32, i32)>,
    search: BruteForceSearch<ThumbInstructionSet>,
    func: fn(i32, i32) -> Option<i32>,
}

impl Aapcs32 {
    /// Returns a new Aapcs32 struct
    pub fn new(
        search: BruteForceSearch<ThumbInstructionSet>,
        func: fn(i32, i32) -> Option<i32>,
    ) -> Self {
        Self {
            inputs: vec![],
            search,
            func,
        }
    }

    fn test1(&self, candidate: &Candidate<Thumb>, a: i32, b: i32) -> bool {
        use crate::Emulator;
        use armv4t_emu::Mode;
        let mut emu = ArmV4T::default();

        if let Some(result) = (self.func)(a, b) {
            emu.set_r0(a);
            emu.set_r1(b);
            emu.run(0x8000, candidate);
            emu.get_r0() == result
        } else {
            true
        }
    }

    fn test(&mut self, candidate: &Candidate<Thumb>) -> bool {
        use rand::random;

        // Try the values that have returned false before
        for inputs in &self.inputs {
            if !self.test1(candidate, inputs.0, inputs.1) {
                return false;
            }
        }

        // Try ten more random value pairs to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a: i32 = random();
            let b: i32 = random();
            if !self.test1(candidate, a, b) {
                self.inputs.push((a, b));
                return false;
            }
        }
        true
    }
}

impl Iterator for Aapcs32 {
    type Item = Candidate<Thumb>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.search.next() {
            if self.test(&candidate) {
                return Some(candidate);
            }
        }
        None
    }
}
