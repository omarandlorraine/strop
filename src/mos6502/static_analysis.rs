//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::mos6502::Instruction6502;
use crate::static_analysis::VarState;
use yaxpeax_6502::{Opcode, Operand};

/// Check for the Y register
pub fn check_use_a(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (Opcode::TAY, _) => state.r(),
        (Opcode::TAX, _) => state.r(),
        (Opcode::CMP, _) => state.r(),
        (Opcode::ADC, _) => state.r().w(),
        (Opcode::SBC, _) => state.r().w(),
        (Opcode::EOR, _) => state.r().w(),
        (Opcode::ORA, _) => state.r().w(),
        (Opcode::AND, _) => state.r().w(),
        (Opcode::LSR, Operand::Accumulator) => state.r().w(),
        (Opcode::ROL, Operand::Accumulator) => state.r().w(),
        (Opcode::ASL, Operand::Accumulator) => state.r().w(),
        (Opcode::ROR, Operand::Accumulator) => state.r().w(),
        (Opcode::LDA, _) => state.w(),
        (Opcode::STA, _) => state.r(),
        (_, _) => state,
    }
}

/// Check for the X register
pub fn check_use_x(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (_, Operand::XIndexedIndirect(_)) => state.r(),
        (_, Operand::ZeroPageX(_)) => state.r(),
        (_, Operand::AbsoluteX(_)) => state.r(),
        (Opcode::TXA, _) => state.r(),
        (Opcode::STX, _) => state.r(),
        (Opcode::CPX, _) => state.r(),
        (Opcode::DEX, _) => state.r(),
        (Opcode::INX, _) => state.r(),
        (Opcode::TXS, _) => state.r(),
        (Opcode::TAX, _) => state.w(),
        (Opcode::LDX, _) => state.w(),
        (_, _) => state,
    }
}

/// Check for the Y register
pub fn check_use_y(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (_, Operand::IndirectYIndexed(_)) => state.r(),
        (_, Operand::ZeroPageY(_)) => state.r(),
        (_, Operand::AbsoluteY(_)) => state.r(),
        (Opcode::TYA, _) => state.r(),
        (Opcode::STY, _) => state.r(),
        (Opcode::CPY, _) => state.r(),
        (Opcode::DEY, _) => state.r(),
        (Opcode::INY, _) => state.r(),
        (Opcode::TAY, _) => state.w(),
        (Opcode::LDY, _) => state.w(),
        (_, _) => state,
    }
}

/// Check for the Carry flag
pub fn check_use_c(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (Opcode::ADC, _) => state.r(),
        (Opcode::ASL, _) => state.w(),
        (Opcode::BCC, _) => state.r(),
        (Opcode::BCS, _) => state.r(),
        (Opcode::CLC, _) => state.w(),
        (Opcode::CMP, _) => state.w(),
        (Opcode::CPX, _) => state.w(),
        (Opcode::CPY, _) => state.w(),
        (Opcode::LSR, _) => state.w(),
        (Opcode::ROL, _) => state.r(),
        (Opcode::ROR, _) => state.r(),
        (Opcode::SBC, _) => state.r(),
        (Opcode::SEC, _) => state.w(),
        (_, _) => state,
    }
}

/// Check for the Decimal flag
pub fn check_use_d(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (Opcode::ADC, _) => state.r(),
        (Opcode::SBC, _) => state.r(),
        (Opcode::CLD, _) => state.w(),
        (Opcode::SED, _) => state.w(),
        (_, _) => state,
    }
}

/// returns true iff the instruction is a conditional branch
fn is_branch(insn: Instruction6502) -> bool {
    match (insn.opcode, insn.operand) {
        (Opcode::BCC, _) => true,
        (Opcode::BCS, _) => true,
        (Opcode::BEQ, _) => true,
        (Opcode::BMI, _) => true,
        (Opcode::BNE, _) => true,
        (Opcode::BPL, _) => true,
        (Opcode::BVC, _) => true,
        (Opcode::BVS, _) => true,
        (_, _) => false,
    }
}
