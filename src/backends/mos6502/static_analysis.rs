use mos6502::Variant;
use crate::Sequence;
use crate::backends::mos6502::Instruction;
use crate::{StaticAnalysis, Fixup};

fn branch<V: Variant>(instruction: &Instruction<V>) -> bool {
    use mos6502::instruction::Instruction;
    matches!(instruction.opcode(), Instruction::BRA |Instruction::BEQ |Instruction::BNE 
    | Instruction::BMI |Instruction::BPL |Instruction::BCC |Instruction::BCS )
}

fn pushes<V: Variant>(instruction: &Instruction<V>) -> bool {
    use mos6502::instruction::Instruction;
    matches!(instruction.opcode(), Instruction::PHP |Instruction::PHA |Instruction::PHX |Instruction::PHY) 
}

fn pulls<V: Variant>(instruction: &Instruction<V>) -> bool {
    use mos6502::instruction::Instruction;
    matches!(instruction.opcode(), Instruction::PLP |Instruction::PLA |Instruction::PLX |Instruction::PLY) 
}

pub fn find_first_php<V: Variant>(subroutine: &Sequence<Instruction<V>>) -> Result<Option<usize>, Fixup<Instruction<V>>> {
    for (offset, instruction) in subroutine.iter().enumerate() {
        use mos6502::instruction::Instruction;
        if matches!(instruction.opcode(), Instruction::PLP |Instruction::PLA |Instruction::PLX |Instruction::PLY) {
            return Err(Fixup::new(
                "stack underflow",
                crate::backends::mos6502::Instruction::skip_opcode,
                offset,
            ));
        }
        if matches!(instruction.opcode(), Instruction::PHP) {
            return Ok(Some(offset));
        }
    }
    Ok(None)
}

/// Static analysis checking that the stack does not overflow. `level` is the number of bytes that
/// the routine may leave on the stack.
pub fn do_not_overflow<V: Variant>(subroutine: &Sequence<Instruction<V>>, limit: u8) -> StaticAnalysis<Instruction<V>> {
    let mut level: u8 = 0;

    for (offset, instruction) in subroutine.iter().enumerate() {
        if branch(&instruction) {
            // Bail out, I haven't thought this through.
            return Ok(());
        }
        if pulls(&instruction) {
            level -= 1;
        }
        if pushes(&instruction) {
            level += 1;
            if level > limit {
            return Err(Fixup::new(
                "stack overflow",
                crate::backends::mos6502::Instruction::skip_opcode,
                offset,
            ));
            }
        }
    }
    Ok(())
}

/// Static analysis checking that the stack does not underflow. `level` is the number of bytes that
/// the routine may pull.
pub fn do_not_underflow<V: Variant>(subroutine: &Sequence<Instruction<V>>, limit: u8) -> StaticAnalysis<Instruction<V>> {
    let mut level: u8 = 0;

    for (offset, instruction) in subroutine.iter().enumerate() {
        if branch(&instruction) {
            // Bail out, I haven't thought this through.
            return Ok(());
        }
        if pulls(&instruction) {
            level += 1;
            if level > limit {
            return Err(Fixup::new(
                "stack underflow",
                crate::backends::mos6502::Instruction::skip_opcode,
                offset,
            ));
            }
        }
        if pushes(&instruction) {
            level -= 1;
        }
    }
    Ok(())
}
