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

use crate::z80::dataflow::Register;
use crate::z80::Insn;
use crate::Constrain;
use crate::DataFlow;
use crate::Sequence;

/// Performs dataflow analysis of immediate register pair loads.
///
/// This is special-cased for reasons explained in the module-level doc comments.
#[derive(Debug)]
pub struct RegPairFixup<'a>(pub &'a mut Sequence<Insn>);

impl RegPairFixup<'_> {
    fn reads(&self, offset: usize, register: &Register) -> bool {
        self.0
            .iter()
            .skip(offset + 1)
            .any(|insn| insn.reads(register))
    }

    fn unnecessary_load(
        &self,
        offset: usize,
        opcode: &[u8],
        left: Register,
        right: Register,
    ) -> bool {
        use crate::Encode;
        let enc = self.0[offset].encode();
        if enc.len() < opcode.len() {
            return false;
        }
        if enc[..opcode.len()] != opcode[..opcode.len()] {
            return false;
        }

        // We're looking at a register pair load. Let's check that both left and right are used by
        // subsequent instructions
        if self.reads(offset, &left) && self.reads(offset, &right) {
            return false;
        }
        true
    }

    fn check(&self, offset: usize) -> Option<&str> {
        if self.unnecessary_load(offset, &[0x01], Register::B, Register::C) {
            Some("No need to load BC, since B and C is not used later")
        } else if self.unnecessary_load(offset, &[0x11], Register::D, Register::E) {
            Some("No need to load DE, since E is not used later")
        } else if self.unnecessary_load(offset, &[0x21], Register::H, Register::L) {
            Some("No need to load HL, since H and L is not used later")
        } else {
            None
        }
    }
}

impl Constrain<Insn> for RegPairFixup<'_> {
    fn fixup(&mut self) {
        for i in 0..(self.0.len() - 1) {
            if self.check(i).is_some() {
                self.0.mut_at(Insn::next_opcode, i);
            }
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        if let Some(r) = self.check(offset) {
            vec![r.to_string()]
        } else {
            vec![]
        }
    }
}
