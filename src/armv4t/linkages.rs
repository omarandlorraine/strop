//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

use crate::Candidate;
use crate::Linkage;
use crate::SearchAlgorithm;

use crate::armv4t::instruction_set::Thumb;

fn check_last_instruction(candidate: &Candidate<Thumb>, instruction: Thumb) -> bool {
    let len = candidate.instructions.len();
    if len < 1 {
        // not long enough to even contain a `ret` instruction or anything.
        return false;
    }
    let offset = len - 1;

    let last_instruction = candidate.instructions[offset];

    last_instruction != instruction
}

fn fixup_last_instruction<S: SearchAlgorithm<Item = Thumb>>(
    search: &mut S,
    candidate: &Candidate<Thumb>,
    instruction: Thumb,
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

/// A type representing the Thumb-encoded interworking subroutine. This is a subroutine which uses
/// the `bx lr` instruction to return. This means that the subroutine will return to either Thumb
/// or ARM code, as appropriate.
#[derive(Debug)]
pub struct InterworkingSubroutine;

impl<S: SearchAlgorithm<Item = Thumb>> Linkage<S, Thumb> for InterworkingSubroutine {
    fn fixup(&self, search: &mut S, candidate: &Candidate<Thumb>) -> bool {
        fixup_last_instruction(search, candidate, Thumb(0x4770))
    }
    fn check(&self, candidate: &Candidate<Thumb>) -> bool {
        check_last_instruction(candidate, Thumb(0x4770))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_instructions_can_be_executed() {
        use crate::armv4t::emulators::ArmV4T;
        use crate::armv4t::Thumb;
        use crate::BruteForceSearch;
        use crate::Emulator;
        use crate::SearchAlgorithm;

        for candidate in BruteForceSearch::<Thumb>::new().iter() {
            if candidate.length() > 1 {
                break; //TODO
            }
            ArmV4T::default().run(0x2000, &candidate);
            candidate.disassemble();
        }
    }
}
