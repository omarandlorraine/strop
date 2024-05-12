//! Backend targeting the ARMv4 CPUs (for example, the ARM7TDMI)

use crate::Fixup;

use crate::armv4t::instruction_set::Thumb;

fn put_back_to(insn: Thumb, should_be: Thumb) -> Option<Thumb> {
    if insn < should_be {
        Some(should_be)
    } else {
        None
    }
}

/// A fixup setting the instruction (back) to `bx lr`. This is the instruction necessary to return
/// from a subroutine, and so a static analysis pass may use this to make sure that the Candidate
/// at least ends in a return instruction.
#[derive(Debug)]
pub struct BxLr;

impl Fixup<Thumb> for BxLr {
    fn random(&self, _insn: Thumb) -> Thumb {
        Thumb(0x4770)
    }

    fn next(&self, insn: Thumb) -> Option<Thumb> {
        put_back_to(insn, Thumb(0x4770))
    }

    fn check(&self, insn: Thumb) -> bool {
        insn != Thumb(0x4770)
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
