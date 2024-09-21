use crate::z80::Insn;
use crate::IterableSequence;
use std::ops::Index;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
#[derive(Clone, Debug)]
pub struct Subroutine(crate::Sequence<Insn>);

impl Subroutine {
    fn fixup(&mut self) {
        use crate::Encode;
        while self.0[self.0.last_instruction_offset()].encode()[0] != 0xc9 {
            // make sure the subroutine ends in a return instruction
            self.0.stride_at(self.0.last_instruction_offset());
        }
    }

    /// Returns the offset of the last instruction in the subroutine
    pub fn last_instruction_offset(&self) -> usize {
        self.0.last_instruction_offset()
    }

    /// Returns the offset of the penultimate instruction in the subroutine, if there is one (i.e.,
    /// the subroutine contains at least one instruction before the `ret` instruction)
    pub fn penultimate_instruction_offset(&self) -> Option<usize> {
        self.last_instruction_offset().checked_sub(1)
    }
}

// Implement the Index trait for read-only access.
impl Index<usize> for Subroutine {
    type Output = Insn;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Default for Subroutine {
    fn default() -> Self {
        Self::new()
    }
}

impl IterableSequence for Subroutine {
    fn first() -> Self {
        Self::new()
    }

    fn stride_at(&mut self, offset: usize) -> bool {
        self.0.stride_at(offset);
        self.fixup();
        true
    }

    fn step_at(&mut self, offset: usize) -> bool {
        self.0.step_at(offset);
        self.fixup();
        true
    }
}

impl Subroutine {
    //! Build a `Subroutine`
    /// Build a `Subroutine`
    pub fn new() -> Self {
        use crate::IterableSequence;

        Self(crate::Sequence::<Insn>::first())
    }
}

impl AsRef<crate::Sequence<Insn>> for Subroutine {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        &self.0
    }
}

impl crate::Goto<Insn> for Subroutine {
    fn goto(&mut self, t: &[Insn]) {
        self.0.goto(t);
    }
}
impl crate::Disassemble for Subroutine {
    fn dasm(&self) {
        self.0.dasm();
    }
}
