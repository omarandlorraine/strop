use crate::z80::Insn;
use crate::Sequence;

/// Makes sure that a `Sequence<Insn>`, that is, a sequence of Z80 instructions, is a valid
/// subroutine, by making sure that it ends in a `RET` instruction.
#[derive(Clone, Debug, PartialEq)]
pub struct Subroutine;

impl crate::Constrain<Insn> for Subroutine {
    fn fixup(&self, candidate: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        if candidate[candidate.len() - 1] != Insn::ret() {
            candidate.mut_at(Insn::next_opcode, candidate.len() - 1);
            return Some((
                candidate.len() - 1,
                "Subroutine not ending in the `RET` instruction",
            ));
        }
        None
    }
}
