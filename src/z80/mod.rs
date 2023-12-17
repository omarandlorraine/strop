//! The Z80 backend (can of course also be used to generate code for the Intel 8080 or the SM83).

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::BruteForceSearch;
use crate::HammingDistance;
use crate::StochasticSearch;
use instruction_set::Z80InstructionSet;
use num::cast::AsPrimitive;
use rand::distributions::Standard;
use rand::prelude::Distribution;

/// Returns the default Z80 instruction set
pub fn z80() -> instruction_set::Z80InstructionSet {
    instruction_set::Z80InstructionSet::default()
}

impl BruteForceSearch<Z80InstructionSet> {
    /// returns an iterator yielding functions complying with the __z88dk_fastcall calling
    /// convention, and computing the provided functions.
    ///
    /// `func` should be a function returning an `Option<i32>`. For inputs where `func` returns
    /// `Some(x)`, the generated function returns `x`. But for inputs where `func` returns `None`,
    /// the behavior of the generated function is undefined.
    pub fn z88dkfastcall<Operand, Return>(
        self,
        func: fn(Operand) -> Option<Return>,
    ) -> testers::Z88dkfastcall<Self, Operand, Return>
    where
        u32: HammingDistance<Return>,
        u32: AsPrimitive<Operand>,
        u32: From<Operand>,
        Standard: Distribution<Operand>,
        Operand: std::marker::Copy + num::traits::AsPrimitive<u32>,
        Return: num::traits::AsPrimitive<u32>,
    {
        testers::Z88dkfastcall::new(self, func)
    }
}

impl StochasticSearch<Z80InstructionSet> {
    /// returns an iterator yielding functions complying with the __z88dk_fastcall calling
    /// convention, and computing the provided functions.
    ///
    /// `func` should be a function returning an `Option<i32>`. For inputs where `func` returns
    /// `Some(x)`, the generated function returns `x`. But for inputs where `func` returns `None`,
    /// the behavior of the generated function is undefined.
    pub fn z88dkfastcall<Operand, Return>(
        self,
        func: fn(Operand) -> Option<Return>,
    ) -> testers::Z88dkfastcall<Self, Operand, Return>
    where
        u32: HammingDistance<Return>,
        u32: AsPrimitive<Operand>,
        u32: From<Operand>,
        Standard: Distribution<Operand>,
        Operand: std::marker::Copy + num::traits::AsPrimitive<u32>,
        Return: num::traits::AsPrimitive<u32>,
    {
        testers::Z88dkfastcall::new(self, func)
    }
}
