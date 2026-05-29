use crate::StaticAnalysis;
use crate::backends::armv4t::Instruction;

pub fn peephole_optimizer(_seq: &crate::Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    Ok(())
}
