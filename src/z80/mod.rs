//! The Z80 backend (can of course also be used to generate code for the Intel 8080 or the SM83).
#![allow(dead_code)] // TODO: enable this lint

pub mod emulators;
pub mod instruction_set;

use crate::SingleInstruction;
use instruction_set::Z80Instruction;



const RET: SingleInstruction<Z80Instruction> = SingleInstruction(Z80Instruction::new([0xc9, 0, 0, 0, 0]));
const RETI: SingleInstruction<Z80Instruction> = SingleInstruction(Z80Instruction::new([0xed, 0x4d, 0, 0, 0]));
const RETN: SingleInstruction<Z80Instruction> = SingleInstruction(Z80Instruction::new([0xed, 0x45, 0, 0, 0]));
