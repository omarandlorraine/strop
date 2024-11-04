use crate::m6502::Insn;
use crate::Sequence;
use std::ops::Index;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
#[derive(Clone, Debug)]
pub struct Subroutine<V: mos6502::Variant + Clone>(crate::Sequence<Insn<V>>);

impl<V: mos6502::Variant + Clone> Subroutine<V> {
    /// Builds the subroutine by concatenating the body of the subroutine with a return
    /// instruction.
    pub fn build(&self) -> Sequence<Insn<V>> {
        vec![&self.0, &vec![Insn::rts()]].into()
    }

    /// Build a `Subroutine`
    pub fn new() -> Self {
        use crate::Iterable;

        Self(crate::Sequence::<Insn<V>>::first())
    }
}

// Implement the Index trait for read-only access.
impl<V: mos6502::Variant + Clone> Index<usize> for Subroutine<V> {
    type Output = Insn<V>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<V: mos6502::Variant + Clone> std::ops::DerefMut for Subroutine<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V: mos6502::Variant + Clone> std::ops::Deref for Subroutine<V> {
    type Target = crate::Sequence<Insn<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V: mos6502::Variant + Clone> Default for Subroutine<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: mos6502::Variant + Clone> AsRef<crate::Sequence<Insn<V>>> for Subroutine<V> {
    fn as_ref(&self) -> &crate::Sequence<Insn<V>> {
        &self.0
    }
}

impl<V: mos6502::Variant + Clone> crate::Goto<Insn<V>> for Subroutine<V> {
    fn goto(&mut self, t: &[Insn<V>]) {
        self.0.goto(t);
    }
}

impl<V: mos6502::Variant + Clone> crate::Disassemble for Subroutine<V>
where
    Insn<V>: crate::Disassemble,
{
    fn dasm(&self) {
        self.0.dasm();
    }
}

impl<V: mos6502::Variant + Clone> crate::Iterable for Subroutine<V> {
    fn first() -> Self {
        Self(crate::Iterable::first())
    }

    fn step(&mut self) -> bool {
        self.0.step()
    }
}
