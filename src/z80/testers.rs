//! Z80 testers.
use crate::HammingDistance;

use crate::SearchFeedback;
use crate::Candidate;
use crate::z80::instruction_set::Z80Instruction;
 
use num::cast::AsPrimitive;
use rand::distributions::Standard;
use rand::prelude::Distribution;

/// Tests the programs visited by a search strategy and yields those which compute the given
/// function and conform to the `__z88dk_fast_call` calling convention (i.e. equivalent to a
/// function created by SDCC using a syntax like
/// ```C
/// int my_abs(int a) __z88dk_fastcall {
///         return (a<0) ? -a : a;
/// }
/// ```
///
/// The function may take exactly one parameter, which is 8, 16 or 32 bits wide and passed in
/// registers DEHL, and may return one value, passed in DEHL. No other registers or flags need to
/// be preserved.
#[derive(Debug)]
pub struct Z88dkfastcall<S, Operand, Return>
where
    S: SearchFeedback,
    S: Iterator<Item = Candidate<Z80Instruction>>,
    Operand: num::cast::AsPrimitive<u32>,
{
    inputs: Vec<(u32, Return)>,
    search: S,
    func: fn(Operand) -> Option<Return>,
}

impl<S: Iterator<Item = Candidate<Z80Instruction>> + SearchFeedback, Operand: num::cast::AsPrimitive<u32>, Return: num::cast::AsPrimitive<u32>> Z88dkfastcall<S, Operand, Return> 
where u32: HammingDistance<Return>, u32: AsPrimitive<Operand>, u32: From<Operand>, Standard: Distribution<Operand>
{
    /// Returns a new Z88dkfastcall struct.
    pub fn new(search: S, func: fn(Operand) -> Option<Return>) -> Self {
        Self {
            inputs: vec![],
            search, func
        }
    }

    fn test1(&self, candidate: &<S as Iterator>::Item, a: u32) -> f32 {
        use crate::Emulator;
        use crate::z80::emulators::Z80;

        if let Some(result) = (self.func)(a.as_()) {
            let mut emu = Z80::default();
            emu.set_dehl(a);
            emu.run(0x8000, candidate);
            emu.get_dehl().hamming_distance(result)
        } else {
            0.0
        }
    }

    fn possible_test_case(
        &mut self,
        candidate: &<S as Iterator>::Item,
        a: Operand
    ) {
        use crate::Emulator;
        use crate::z80::emulators::Z80;

        if let Some(result) = (self.func)(a) {
            let mut emu = Z80::default();
            emu.set_dehl(a.into());
            emu.run(0x8000, candidate);
            if emu.get_dehl().hamming_distance(result) != 0.0 {
                self.inputs.push((a.into(), result));
            }
        }
    }

    fn correctness(&self, candidate: &Candidate<Z80Instruction>) -> f32 {
        let mut score = 0.0;
        // Try the values that have returned false before
        for inputs in &self.inputs {
            score += self.test1(candidate, inputs.0);
        }
        score
    }

    fn test(&mut self, candidate: &Candidate<Z80Instruction>) -> f32 {
        use rand::random;

        let mut score = self.correctness(candidate);
        if score > 0.0 {
            return score;
        }

        // Try ten more random value pairs across a small range to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a: Operand = random();
            self.possible_test_case(candidate, a);
            score += self.test1(candidate, a.into());
        }
        score
    }

}

impl<S: Iterator<Item = Candidate<Z80Instruction>> + SearchFeedback, Operand: num::cast::AsPrimitive<u32>, Return> Iterator for Z88dkfastcall<S, Operand, Return> 
where u32: HammingDistance<Return>, u32: AsPrimitive<Operand>, u32: From<Operand>, Standard: Distribution<Operand> 
    , Return: num::cast::AsPrimitive<u32>
    {
    type Item = Candidate<Z80Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.search.next() {
            let score = self.test(&candidate);
            self.search.score(score as f32);
            if score == 0.0 {
                // We've found a program that passes the test cases we've found; let's optimize the
                // program.
                return Some(candidate);
            }
        }
        None
    }
}
