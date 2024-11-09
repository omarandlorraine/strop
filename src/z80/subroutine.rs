use crate::z80::Insn;
use std::ops::Index;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
#[derive(Clone, Debug)]
pub struct Subroutine(crate::Sequence<Insn>);

impl crate::Constrain<Insn> for Subroutine {
    fn fixup(&mut self) {
        while self.0[self.0.len() - 1] != Insn::ret() {
            self.0.mut_at(Insn::next_opcode, self.0.len() - 1)
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        if offset != self.0.len() - 1 {
            vec![]
        } else if self.0[offset] != Insn::ret() {
            vec!["Subroutine not ending in the `RET` instruction".to_string()]
        } else {
            vec![]
        }
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

impl crate::Iterable for Subroutine {
    fn first() -> Self {
        Self(crate::Iterable::first())
    }

    fn step(&mut self) -> bool {
        self.0.step()
    }
}
