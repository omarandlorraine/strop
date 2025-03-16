//! A module defining Subroutine<T>

/// A type representing a subroutine. This includes the static analysis to make sure that the
/// instruction sequence ends in the appropriate return instruction, etc.
#[derive(Debug, Clone)]
pub struct Subroutine<S>(S);

pub trait ShouldReturn {
    fn should_return(&self) -> Option<crate::StaticAnalysis<Self>> where Self: Sized;
}

impl<S> Subroutine<S> {
    /// Wraps the object in the Subroutine struct
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

impl<S: AsRef<crate::Sequence<Insn>>, Insn: ShouldReturn> crate::Analyse<Insn> for Subroutine<S> {
    fn analyse(&self) -> Option<crate::StaticAnalysis<Insn>> {
        let seq = self.0.as_ref();
        seq.sa(seq.last_instruction_offset(), Insn::should_return)
    }
}

pub trait ToSubroutine<T: ShouldReturn> {
    fn to_subroutine(self) -> Subroutine<Self>
    where
        Self: Sized,
    {
        Subroutine::<Self>::new(self)
    }
}

impl<I, T: crate::Goto<I>> crate::Goto<I> for Subroutine<T> {
    fn goto(&mut self, code: &[I]) {
        self.0.goto(code);
    }
}

impl<T: crate::Disassemble> crate::Disassemble for Subroutine<T> {
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl<S: crate::Step> crate::Step for Subroutine<S> {
    fn first() -> Self {
        Self(S::first())
    }
    fn next(&mut self) -> crate::IterationResult {
        self.0.next()
    }
}

impl<S: crate::Encode<E>, E> crate::Encode<E> for Subroutine<S> {
    fn encode(&self) -> Vec<E> {
        self.0.encode()
    }
}

impl<T> AsMut<T> for Subroutine<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> AsRef<T> for Subroutine<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<D, T: crate::dataflow::DataFlow<D>> crate::dataflow::DataFlow<D> for Subroutine<T> {
    fn reads(&self, t: &D) -> bool {
        self.0.reads(t)
    }
    fn writes(&self, t: &D) -> bool {
        self.0.writes(t)
    }
    fn modify(&mut self) -> crate::IterationResult {
        self.0.modify()
    }
}
