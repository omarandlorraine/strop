use crate::Instruction as _;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::backends::mips::Instruction;
use trapezoid_core::cpu::RegisterType;

fn skip_this_opcode() -> StaticAnalysis<Instruction> {
    Err(crate::Fixup::<Instruction> {
        advance: |i| {
            if i.r() {
                i.next_opcode()
            } else {
                i.increment()
            }
        },
        offset: 0,
        reason: "RedundantEncoding",
    })
}

fn rd_should_not_be_zero(insn: &Instruction) -> StaticAnalysis<Instruction> {
    if insn.rd().unwrap() == RegisterType::Zero {
        skip_this_opcode()
    } else {
        Ok(())
    }
}

fn rs_should_not_be_zero(insn: &Instruction) -> StaticAnalysis<Instruction> {
    if insn.rs().unwrap() == RegisterType::Zero {
        skip_this_opcode()
    } else {
        Ok(())
    }
}

fn rs_should_not_equal_rd(insn: &Instruction) -> StaticAnalysis<Instruction> {
    if insn.rs() == insn.rd() {
        skip_this_opcode()
    } else {
        Ok(())
    }
}

fn shamt_should_be_greater_than_zero(insn: &Instruction) -> StaticAnalysis<Instruction> {
    if insn.shamt().unwrap() == 0 {
        skip_this_opcode()
    } else {
        Ok(())
    }
}

/// This looks at a single instruction for pointless things.
fn check_for_pointless_instructions(insn: &Instruction) -> StaticAnalysis<Instruction> {
    use trapezoid_core::cpu::Opcode;

    match insn.decode().opcode {
        // There's no point generating NOPs
        Opcode::Nop => skip_this_opcode(),
        Opcode::Sll => {
            // Shifting left by zero bits is ineffectual
            shamt_should_be_greater_than_zero(insn)?;
            Ok(())
        }
        Opcode::Sllv => {
            // Shifting left by zero bits is ineffectual
            rs_should_not_be_zero(insn)?;
            Ok(())
        }
        Opcode::Srlv => {
            // Shifting right by zero bits is ineffectual
            rs_should_not_be_zero(insn)?;
            Ok(())
        }
        Opcode::Srav => {
            // Shifting right by zero bits is ineffectual
            rs_should_not_be_zero(insn)?;
            Ok(())
        }
        Opcode::Srl => {
            // Shifting right by zero bits is ineffectual
            shamt_should_be_greater_than_zero(insn)?;
            Ok(())
        }
        Opcode::Sra => {
            // Shifting right by zero bits is ineffectual
            shamt_should_be_greater_than_zero(insn)?;
            Ok(())
        }
        Opcode::Multu => {
            // multiplying by zero is silly
            rs_should_not_be_zero(insn)?;
            // don't think there's much point storing the result in $zero either
            rd_should_not_be_zero(insn)?;
            Ok(())
        }
        Opcode::Jr => {
            // no point trying to jump to $zero
            rs_should_not_be_zero(insn)?;
            Ok(())
        }
        Opcode::Jalr => {
            // no point trying to jump to $zero
            rd_should_not_be_zero(insn)?;
            // also no point trying to store the return value in $zero
            rs_should_not_be_zero(insn)?;
            // also if $rs and $rd are the same then the instruction is UNPREDICTABLE
            rs_should_not_equal_rd(insn)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn peephole_optimizer(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    seq.check_all(check_for_pointless_instructions)
}
