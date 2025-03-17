//! A module defining Subroutine<T>

use crate::BruteforceSearch;
use crate::Sequence;

/// A type representing a subroutine. This includes the static analysis to make sure that the
/// instruction sequence ends in the appropriate return instruction, etc.
#[derive(Debug, Clone)]
pub struct Subroutine<Insn, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>(
    S,
    std::marker::PhantomData<Insn>,
);

pub trait ShouldReturn {
    fn should_return(&self) -> Option<crate::StaticAnalysis<Self>>
    where
        Self: Sized;
}

impl<Insn, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> AsRef<Sequence<Insn>>
    for Subroutine<Insn, S>
{
    fn as_ref(&self) -> &Sequence<Insn> {
        self.0.as_ref()
    }
}

impl<Insn, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> Subroutine<Insn, S> {
    /// Wraps the object in the Subroutine struct
    pub fn new(s: S) -> Self {
        Self(s, std::marker::PhantomData::default())
    }
}

impl<Insn: ShouldReturn + std::fmt::Debug, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    BruteforceSearch<Insn> for Subroutine<Insn, S>
{
    fn analyze_this(&self) -> Option<crate::StaticAnalysis<Insn>>
    where
        Self: Sized,
    {
        let seq = self.0.as_ref();
        seq[seq.len() - 1].should_return()
    }

    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.0
    }
}

pub trait ToSubroutine<T: ShouldReturn> {
    fn to_subroutine(self) -> Subroutine<T, Self>
    where
        Self: Sized + BruteforceSearch<T>,
        Self: AsRef<Sequence<T>>,
    {
        Subroutine::<T, Self>::new(self)
    }
}

impl<Insn, S: crate::Disassemble + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    crate::Disassemble for Subroutine<Insn, S>
{
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl<Insn, S: crate::Step + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> crate::Step
    for Subroutine<Insn, S>
{
    fn first() -> Self {
        Self(S::first(), std::marker::PhantomData::default())
    }
    fn next(&mut self) -> crate::IterationResult {
        self.0.next()
    }
}

impl<Insn, S: crate::Encode<E> + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>, E> crate::Encode<E>
    for Subroutine<Insn, S>
{
    fn encode(&self) -> Vec<E> {
        self.0.encode()
    }
}

impl<Insn, T: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> AsMut<T> for Subroutine<Insn, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<Insn, D, T: crate::dataflow::DataFlow<D> + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    crate::dataflow::DataFlow<D> for Subroutine<Insn, T>
{
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
