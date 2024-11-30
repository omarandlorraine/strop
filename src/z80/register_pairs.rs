//! Because immediate loads of register pairs take so much search space (that is, the average
//! bruteforce search needs to traipse through a lot of these instructions to find anything
//! useful), we need a way to prune these from the search.
//!
//! The usual dataflow analysis leaves these instructions alone because they load register pairs,
//! and because the dataflow analysis works on single registers at a time, and thus can't prove
//! that these instructions are dead code.
//!
//! If a program contains, `LD BC, 1234h`, then I am fixing this up unless *both* `B` and `C` are
//! used afterwards.
//!
//! Therefore, this module implements conveniences for skipping these instructions.

use crate::dataflow::DataFlow;
use crate::z80::dataflow::Register;
use crate::z80::Insn;
use crate::Constrain;
use crate::Sequence;

/// Performs dataflow analysis of immediate register pair loads.
///
/// This is special-cased for reasons explained in the module-level doc comments.
#[derive(Debug)]
pub struct RegPairFixup();

impl RegPairFixup {
    fn reads(&self, candidate: &Sequence<Insn>, offset: usize, register: &Register) -> bool {
        candidate
            .iter()
            .skip(offset + 1)
            .any(|insn| insn.reads(register))
    }

    fn unnecessary_load(
        &self,
        candidate: &Sequence<Insn>,
        offset: usize,
        opcode: &[u8],
        left: Register,
        right: Register,
    ) -> bool {
        use crate::Encode;
        let enc = candidate[offset].encode();
        if enc.len() < opcode.len() {
            return false;
        }
        if enc[..opcode.len()] != opcode[..opcode.len()] {
            return false;
        }

        // We're looking at a register pair load. Let's check that both left and right are used by
        // subsequent instructions
        if self.reads(candidate, offset, &left) && self.reads(candidate, offset, &right) {
            return false;
        }
        true
    }

    fn check(&self, candidate: &Sequence<Insn>, offset: usize) -> Option<&'static str> {
        if self.unnecessary_load(candidate, offset, &[0x01], Register::B, Register::C) {
            Some("No need to load BC, since B and C is not used later")
        } else if self.unnecessary_load(candidate, offset, &[0x11], Register::D, Register::E) {
            Some("No need to load DE, since E is not used later")
        } else if self.unnecessary_load(candidate, offset, &[0x21], Register::H, Register::L) {
            Some("No need to load HL, since H and L is not used later")
        } else {
            None
        }
    }
}

impl Constrain<Insn> for RegPairFixup {
    fn fixup(&self, candidate: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        for i in 0..(candidate.len() - 1) {
            if let Some(r) = self.check(candidate, i) {
                candidate.mut_at(Insn::next_opcode, i);
                return Some((i, r));
            }
        }
        None
    }
}
