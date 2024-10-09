use crate::z80::Insn;
use crate::Sequence;
use std::ops::Index;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
#[derive(Clone, Debug)]
pub struct Subroutine(crate::Sequence<Insn>);

impl Subroutine {
    /// Builds the subroutine by concatenating the body of the subroutine with a return
    /// instruction.
    pub fn build(&self) -> Sequence<Insn> {
        vec![&self.0, &vec![Insn::new(&[0xc9])]].into()
    }
}

// Implement the Index trait for read-only access.
impl Index<usize> for Subroutine {
    type Output = Insn;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::DerefMut for Subroutine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Subroutine {
    type Target = crate::Sequence<Insn>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Subroutine {
    fn default() -> Self {
        Self::new()
    }
}

impl Subroutine {
    //! Build a `Subroutine`
    /// Build a `Subroutine`
    pub fn new() -> Self {
        use crate::Iterable;

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
