//! Z80 testers.
use crate::z80::instruction_set::Z80Instruction;
use crate::Candidate;
use crate::Scalar;
use crate::SearchAlgorithm;
use crate::Fixup;

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
pub struct Z88dkFastCall<S, Operand, Return>
where
    S: SearchAlgorithm<Item = Z80Instruction>,
    Operand: num::cast::AsPrimitive<u32>,
{
    func: fn(Operand) -> Option<Return>,
    inputs: Vec<(Operand, Return)>,
    search: S
}

impl<
        S: SearchAlgorithm<Item = Z80Instruction>,
        Operand: num::cast::AsPrimitive<u32>,
        Return: num::cast::AsPrimitive<u32>,
    > Z88dkFastCall<S, Operand, Return>
where
    Return: Scalar,
    Operand: Scalar,
{
    /// Returns a new Z88dkFastCall struct.
    pub fn new(search: S, func: fn(Operand) -> Option<Return>) -> Self {
        Self {
            inputs: vec![],
            search,
            func,
        }
    }

    fn test1(&self, candidate: &Candidate<Z80Instruction>, a: Operand) -> f32 {
        use crate::z80::emulators::Z80;

        if let Some(result) = (self.func)(a) {
            let mut emu = Z80::default();
            emu.set_dehl(a.as_i32());
            emu.set_sp(0x3000);
            emu.run_subroutine(0x8000, 0x4000, candidate);
            (emu.get_dehl().hamming(result)
                + emu.get_sp().hamming(0x3000)
                + emu.get_pc().hamming(0x4003)) as f32
        } else {
            0.0
        }
    }

    fn possible_test_case(&mut self, candidate: &Candidate<Z80Instruction>, a: Operand) {
        use crate::z80::emulators::Z80;
        use crate::Emulator;

        if let Some(result) = (self.func)(a) {
            let mut emu = Z80::default();
            emu.set_dehl(a);
            emu.run(0x8000, candidate);
            if emu.get_dehl().hamming(result) != 0 {
                self.inputs.push((a, result));
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
        let mut score = self.correctness(candidate);
        if score > 0.0 {
            return score;
        }

        // Try ten more random value pairs across a small range to see if we discover any other values where the
        // function returns something different from the generated program
        for _ in 0..10 {
            let a = Operand::random();
            self.possible_test_case(candidate, a);
            score += self.test1(candidate, a);
        }
        score
    }
}

impl<S: SearchAlgorithm<Item = Z80Instruction>, Operand: num::cast::AsPrimitive<u32>, Return>
    SearchAlgorithm for Z88dkFastCall<S, Operand, Return>
where
    Return: Scalar,
    Operand: Scalar,
{
    type Item = Z80Instruction;

    fn score(&mut self, score: f32) {
        self.search.score(score);
    }

    fn replace<F: Fixup<Self::Item>>(&mut self, offset: usize, fixup: F) -> bool {
        self.search.replace(offset, fixup)
    }

    fn generate(&mut self) -> Option<Candidate<Self::Item>> {
        while let Some(candidate) = self.search.generate() {
            if !self.search.replace(candidate.last_offset(), crate::z80::RET) {
                let score = self.test(&candidate);
                self.search.score(score);
                if score == 0.0 {
                    // We've found a program that passes the test cases we've found; let's optimize the
                    // program.
                    return Some(candidate);
                }
            }
        }
        None
    }
}
