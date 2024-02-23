//! The Z80 backend (can of course also be used to generate code for the Intel 8080 or the SM83).

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::BruteForceSearch;
use crate::Candidate;
use crate::Compatibility;
use crate::CompatibilitySearch;
use crate::HammingDistance;
use crate::Linkage;
use crate::LinkageSearch;
use crate::SearchAlgorithm;
use crate::SearchCull;
use crate::StochasticSearch;
use instruction_set::Z80Instruction;
use num::cast::AsPrimitive;
use rand::distributions::Standard;
use rand::prelude::Distribution;

macro_rules! z80impl {
    ($t:ty) => {
        impl $t {
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
    };
}

z80impl!(StochasticSearch<Z80Instruction>);
z80impl!(BruteForceSearch<Z80Instruction>);
z80impl!(
    LinkageSearch<
        CompatibilitySearch<StochasticSearch<Z80Instruction>, Z80Instruction, ZilogZ80>,
        Z80Instruction,
        Subroutine,
    >
);
z80impl!(CompatibilitySearch<StochasticSearch<Z80Instruction>, Z80Instruction, ZilogZ80>);

/// A type representing the Zilog Z80. Useful for a `CompatibilitySearch` for example.
#[derive(Debug)]
pub struct ZilogZ80;

impl Compatibility<Z80Instruction> for ZilogZ80 {
    fn check(&self, _instruction: &Z80Instruction) -> SearchCull<Z80Instruction> {
        SearchCull::Okay
    }
}

/// A type representing the Intel 8080. Useful for a `CompatibilitySearch` for example.
#[derive(Debug)]
pub struct Intel8080;

impl Compatibility<Z80Instruction> for Intel8080 {
    fn check(&self, instruction: &Z80Instruction) -> SearchCull<Z80Instruction> {
        use crate::Instruction;

        let opcode = instruction.encode()[0];
        if matches!(
            opcode,
            0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xd9 | 0xcb | 0xed | 0xdd | 0xfd
        ) {
            // So the opcode is not valid on the Intel 8080. Suggest the next opcode.
            return SearchCull::SkipTo(Some(Z80Instruction::new([opcode + 1, 0, 0, 0, 0])));
        }

        SearchCull::Okay
    }
}

fn check_last_instruction(
    candidate: &Candidate<Z80Instruction>,
    instruction: Z80Instruction,
) -> bool {
    let len = candidate.instructions.len();
    if len < 1 {
        // not long enough to even contain a `ret` instruction or anything.
        return false;
    }
    let offset = len - 1;

    let last_instruction = candidate.instructions[offset];

    last_instruction != instruction
}

fn fixup_last_instruction<S: SearchAlgorithm<Item = Z80Instruction>>(
    search: &mut S,
    candidate: &Candidate<Z80Instruction>,
    instruction: Z80Instruction,
) -> bool {
    let len = candidate.instructions.len();
    if len < 1 {
        // not long enough to even contain a `ret` instruction or anything.
        return false;
    }
    let offset = len - 1;

    let last_instruction = candidate.instructions[offset];

    if last_instruction < instruction {
        search.replace(offset, Some(instruction));
        false
    } else if last_instruction > instruction {
        search.replace(offset, None);
        false
    } else {
        true
    }
}

/// A type representing the Z80 subroutine. The `Linkage` trait is implemented here, so use this if
/// you want to search only for subroutines ending in the `ret` instruction. As per an ordinary Z80
/// subroutine.
#[derive(Debug)]
pub struct Subroutine;

impl<S: SearchAlgorithm<Item = Z80Instruction>> Linkage<S, Z80Instruction> for Subroutine {
    fn fixup(&self, search: &mut S, candidate: &Candidate<Z80Instruction>) -> bool {
        fixup_last_instruction(search, candidate, Z80Instruction::new([0xc9, 0, 0, 0, 0]))
    }

    fn check(&self, candidate: &Candidate<Z80Instruction>) -> bool {
        check_last_instruction(candidate, Z80Instruction::new([0xc9, 0, 0, 0, 0]))
    }
}

/// A type representing the IRQ handler. The `Linkage` trait is implemented here, so use this if
/// you want to search only for subroutines ending in the `reti` instruction. As per an ordinary Z80
/// IRQ handler.
#[derive(Debug)]
pub struct IrqHandler;

impl<S: SearchAlgorithm<Item = Z80Instruction>> Linkage<S, Z80Instruction> for IrqHandler {
    fn fixup(&self, search: &mut S, candidate: &Candidate<Z80Instruction>) -> bool {
        fixup_last_instruction(
            search,
            candidate,
            Z80Instruction::new([0xed, 0x4d, 0, 0, 0]),
        )
    }
    fn check(&self, candidate: &Candidate<Z80Instruction>) -> bool {
        check_last_instruction(candidate, Z80Instruction::new([0xed, 0x4d, 0, 0, 0]))
    }
}

/// A type representing the NMI handler. The `Linkage` trait is implemented here, so use this if
/// you want to search only for subroutines ending in the `retn` instruction. As per an ordinary Z80
/// NMI handler.
#[derive(Debug)]
pub struct NmiHandler;

impl<S: SearchAlgorithm<Item = Z80Instruction>> Linkage<S, Z80Instruction> for NmiHandler {
    fn fixup(&self, search: &mut S, candidate: &Candidate<Z80Instruction>) -> bool {
        fixup_last_instruction(
            search,
            candidate,
            Z80Instruction::new([0xed, 0x45, 0, 0, 0]),
        )
    }
    fn check(&self, candidate: &Candidate<Z80Instruction>) -> bool {
        check_last_instruction(candidate, Z80Instruction::new([0xed, 0x45, 0, 0, 0]))
    }
}
