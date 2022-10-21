//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]



use crate::mos6502::Instruction6502;
use crate::static_analysis::VarState;
use yaxpeax_6502::{Opcode, Operand};

/// Check for the X register
pub fn check_use_x(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (_, Operand::XIndexedIndirect(_)) => state.used(),
        (_, Operand::ZeroPageX(_)) => state.used(),
        (_, Operand::AbsoluteX(_)) => state.used(),
        (Opcode::TXA, _) => state.used(),
        (Opcode::STX, _) => state.used(),
        (Opcode::CPX, _) => state.used(),
        (Opcode::DEX, _) => state.used(),
        (Opcode::INX, _) => state.used(),
        (Opcode::TXS, _) => state.used(),
        (Opcode::TAX, _) => state.init(),
        (Opcode::LDX, _) => state.init(),
        (_, _) => state,
    }
}

/// Check for the Y register
pub fn check_use_y(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (_, Operand::IndirectYIndexed(_)) => state.used(),
        (_, Operand::ZeroPageY(_)) => state.used(),
        (_, Operand::AbsoluteY(_)) => state.used(),
        (Opcode::TYA, _) => state.used(),
        (Opcode::STY, _) => state.used(),
        (Opcode::CPY, _) => state.used(),
        (Opcode::DEY, _) => state.used(),
        (Opcode::INY, _) => state.used(),
        (Opcode::TAY, _) => state.init(),
        (Opcode::LDY, _) => state.init(),
        (_, _) => state,
    }
}

/// Check for the Carry flag
pub fn check_use_c(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (Opcode::ADC, _) => state.used(),
        (Opcode::ASL, _) => state.init(),
        (Opcode::BCC, _) => state.used(),
        (Opcode::BCS, _) => state.used(),
        (Opcode::CLC, _) => state.init(),
        (Opcode::CMP, _) => state.init(),
        (Opcode::CPX, _) => state.init(),
        (Opcode::CPY, _) => state.init(),
        (Opcode::LSR, _) => state.init(),
        (Opcode::ROL, _) => state.used(),
        (Opcode::ROR, _) => state.used(),
        (Opcode::SBC, _) => state.used(),
        (Opcode::SEC, _) => state.init(),
        (_, _) => state,
    }
}

/// Check for the Decimal flag
pub fn check_use_d(state: VarState, insn: &Instruction6502) -> VarState {
    match (insn.opcode, insn.operand) {
        (Opcode::ADC, _) => state.used(),
        (Opcode::SBC, _) => state.used(),
        (Opcode::CLD, _) => state.init(),
        (Opcode::SED, _) => state.init(),
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
