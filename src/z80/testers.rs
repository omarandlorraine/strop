//! Z80 testers.
use crate::HammingDistance;

use crate::z80::instruction_set::Z80Instruction;
use crate::Candidate;
use crate::SearchAlgorithm;

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
    S: SearchAlgorithm<Z80Instruction>,
    Operand: num::cast::AsPrimitive<u32>,
{
    inputs: Vec<(u32, Return)>,
    search: S,
    func: fn(Operand) -> Option<Return>,
}

impl<
        S: SearchAlgorithm<Z80Instruction>,
        Operand: num::cast::AsPrimitive<u32>,
        Return: num::cast::AsPrimitive<u32>,
    > Z88dkfastcall<S, Operand, Return>
where
    u32: HammingDistance<Return>,
    u32: AsPrimitive<Operand>,
    u32: From<Operand>,
    Standard: Distribution<Operand>,
{
    /// Returns a new Z88dkfastcall struct.
    pub fn new(search: S, func: fn(Operand) -> Option<Return>) -> Self {
        Self {
            inputs: vec![],
            search,
            func,
        }
    }

    fn test1(&self, candidate: &Candidate<Z80Instruction>, a: u32) -> f32 {
        use crate::z80::emulators::Z80;

        if let Some(result) = (self.func)(a.as_()) {
            let mut emu = Z80::default();
            emu.set_dehl(a);
            emu.set_sp(0x3000);
            emu.run_subroutine(0x8000, 0x4000, candidate);
            emu.get_dehl().hamming_distance(result)
                + emu.get_sp().hamming_distance(0x3000)
                + emu.get_pc().hamming_distance(0x4003)
        } else {
            0.0
        }
    }

    fn possible_test_case(&mut self, candidate: &Candidate<Z80Instruction>, a: Operand) {
        use crate::z80::emulators::Z80;
        use crate::Emulator;

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

    fn optimize(&self, candidate: &Candidate<Z80Instruction>) -> Candidate<Z80Instruction> {
        use crate::search::DeadCodeEliminator;
        let mut optimizer = DeadCodeEliminator::new(candidate);
        let mut optimized = candidate.clone();

        for _ in 0..1000 {
            // try removing a bajillion instructions at random.
            let candidate = optimizer
                .generate()
                .expect("The dead code eliminator is broken! Why has it stopped trying!");
            let score = self.correctness(&candidate);
            if score == 0.0 {
                optimized = candidate;
            }
            optimizer.score(score);
        }
        optimized
    }
}

impl<
        S: Iterator<Item = Candidate<Z80Instruction>> + SearchAlgorithm<Z80Instruction>,
        Operand: num::cast::AsPrimitive<u32>,
        Return,
    > Iterator for Z88dkfastcall<S, Operand, Return>
where
    u32: HammingDistance<Return>,
    u32: AsPrimitive<Operand>,
    u32: From<Operand>,
    Standard: Distribution<Operand>,
    Return: num::cast::AsPrimitive<u32>,
{
    type Item = Candidate<Z80Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(candidate) = self.search.next() {
            let score = self.test(&candidate);
            self.search.score(score);
            if score == 0.0 {
                // We've found a program that passes the test cases we've found; let's optimize the
                // program.
                return Some(self.optimize(&candidate));
            }
        }
        None
    }
}
