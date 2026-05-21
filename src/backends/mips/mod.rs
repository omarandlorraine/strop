//! A strop backend targetting very basic MIPS processors

mod bus;
mod instruction_set;
pub use instruction_set::Instruction;
pub mod o32;

/// Returns a ExplorationPoint, of MIPS instructions, which makes sure to end in a return
/// instruction. Makes sure that relative branches are in range, and to avoid add/subtract
/// instructions with bounds checking.
pub fn subroutine() -> crate::search::platform_specific::ExplorationPoint<Instruction> {
    crate::search::platform_specific::ExplorationPoint::<Instruction>::new()
        .add(|seq| seq.check_last(Instruction::make_jr_ra))
        .add(|seq| seq.check_all(Instruction::no_overflow_exceptions))
        .add(check_branches_are_in_range)
        .to_owned()
}

#[cfg(test)]
mod test;

/// Makes sure that the relative branches in a sequence don't point outside of the sequence, or to
/// themselvers, or to the following instruction
pub fn check_branches_are_in_range(
    seq: &crate::Sequence<Instruction>,
) -> crate::StaticAnalysis<Instruction> {
    if seq.is_empty() {
        // empty sequence - it can't contain a branch
        return Ok(());
    }

    // Address of the last instruction in the sequence; a branch shouldn't target anything above
    // this.
    let last = (seq.to_bytes().len() - 1) as u32;

    let Some(offset) = seq
        .iter()
        .enumerate()
        .map(|(offset, instruction)| (offset, offset.wrapping_mul(4) as u32, instruction))
        .filter(|(_offset, _pc, instruction)| instruction.is_relative_branch())
        .map(|(offset, pc, instruction)| {
            (
                offset,
                pc,
                pc.wrapping_add((instruction.imm().unwrap() as i16 as i32 as u32).wrapping_mul(4))
                    .wrapping_add(4),
            )
        })
        .filter(|(_offset, pc, destination)|
            // the branch target is outside of the instruction sequence
            *destination > last
            // or the branch targets itself
            || *destination == *pc
            // or the branch targets the following instruction
            || *destination == *pc + 4)
        .map(|(offset, _pc, _destination)| offset)
        .next()
    else {
        return Ok(());
    };

    Err(crate::Fixup::new(
        "BranchOutOfRange",
        crate::Instruction::increment,
        offset,
    ))
}
