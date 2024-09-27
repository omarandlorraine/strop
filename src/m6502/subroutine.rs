use crate::m6502::Insn;
use crate::IterableSequence;
use std::ops::Index;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
#[derive(Clone, Debug)]
pub struct Subroutine<V: mos6502::Variant + std::clone::Clone>(crate::Sequence<Insn<V>>);

impl<V: mos6502::Variant + std::clone::Clone> Subroutine<V> {
    fn fixup(&mut self) {
        use crate::Encode;
        while self.0[self.0.last_instruction_offset()].encode()[0] != 0x60 {
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
impl<V: mos6502::Variant + std::clone::Clone> Index<usize> for Subroutine<V> {
    type Output = Insn<V>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<V: mos6502::Variant + std::clone::Clone> Default for Subroutine<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: mos6502::Variant + std::clone::Clone> IterableSequence for Subroutine<V> {
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

impl<V: mos6502::Variant + std::clone::Clone> Subroutine<V> {
    //! Build a `Subroutine`
    /// Build a `Subroutine`
    pub fn new() -> Self {
        use crate::IterableSequence;

        Self(crate::Sequence::<Insn<V>>::first())
    }
}

impl<V: mos6502::Variant + std::clone::Clone> AsRef<crate::Sequence<Insn<V>>> for Subroutine<V> {
    fn as_ref(&self) -> &crate::Sequence<Insn<V>> {
        &self.0
    }
}

impl<V: mos6502::Variant + std::clone::Clone> crate::Goto<Insn<V>> for Subroutine<V> {
    fn goto(&mut self, t: &[Insn<V>]) {
        self.0.goto(t);
    }
}

impl<V: mos6502::Variant + std::clone::Clone> crate::Disassemble for Subroutine<V>
where
    Insn<V>: crate::Disassemble,
{
    fn dasm(&self) {
        self.0.dasm();
    }
}
