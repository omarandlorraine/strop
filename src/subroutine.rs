//! A module defining Subroutine<T>

use crate::Sequence;
use crate::StaticAnalysis;

pub trait ShouldReturn {
    fn should_return(&self, offset: usize) -> Result<(), crate::StaticAnalysis<Self>>
    where
        Self: Sized;

    fn allowed_in_subroutine(&self) -> Result<(), crate::StaticAnalysis<Self>>
    where
        Self: Sized,
    {
        Ok(())
    }
}

pub fn make_return<Insn: ShouldReturn>(
    sequence: &Sequence<Insn>,
) -> Result<(), StaticAnalysis<Insn>> {
    let offs = sequence.last_instruction_offset();
    sequence[offs].should_return(offs)
}

pub fn not_allowed_in_subroutine<Insn: ShouldReturn>(
    sequence: &Sequence<Insn>,
) -> Result<(), StaticAnalysis<Insn>> {
    for (offs, insn) in sequence.iter().enumerate() {
        if let Err(e) = insn.allowed_in_subroutine() {
            return Err(e.set_offset(offs));
        }
    }
    Ok(())
}

pub fn branches_in_range<Insn: crate::Branch + crate::Encode<u8>>(
    sequence: &Sequence<Insn>,
) -> Result<(), crate::StaticAnalysis<Insn>> {
    // Make a note of the start addresses of all instructions in the subroutine
    let start_addresses = sequence
        .iter()
        .map(|insn| insn.len())
        .scan(0, |sum, x| {
            *sum += x;
            Some(*sum as isize)
        })
        .collect::<Vec<isize>>();

    // Make sure all branches target an actual instruction in the subroutine (that is,
    // disallow instructions that jump out of the subroutine, or that jump to the middle of an
    // instruction)
    let mut backward = 0;
    for insn in sequence.iter() {
        let permissibles = start_addresses
            .iter()
            .flat_map(|x| x.checked_sub(backward))
            .collect::<Vec<isize>>();
        insn.branch_fixup(&permissibles)?;
        backward += insn.len() as isize;
    }

    Ok(())
}

pub fn std_subroutine<Insn: crate::Branch + crate::Encode<u8> + ShouldReturn>(
    sequence: &Sequence<Insn>,
) -> Result<(), crate::StaticAnalysis<Insn>> {
    make_return(sequence)?;
    branches_in_range(sequence)?;
    not_allowed_in_subroutine(sequence)?;
    Ok(())
}
