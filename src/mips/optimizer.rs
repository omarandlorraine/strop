use crate::Sequence;
use crate::StaticAnalysis;
/// If the sequence contains any instruction that I've deemed pointless, then this function returns
/// a StaticAnalysis incrementing the instruction word. An example of an instruction which I've
/// deemed pointless would be `sll v0, a0, 0x00`, since this encodes essentially the same operation
/// as `or v0, a0, $zero` and others.
use crate::mips::Insn;
use crate::static_analysis::Fixup;

pub fn skip_pointless_instructions(sequence: &Sequence<Insn>) -> StaticAnalysis<Insn> {
    for (offs, insn) in sequence.iter().enumerate() {
        Fixup::check(!insn.pointless(), "PointlessInstruction", crate::search::Instruction::increment, offs)?;
    }
    Ok(())
}
