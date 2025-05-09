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

impl<Insn: crate::Step + Clone + ShouldReturn> Default for Subroutine<Insn, Sequence<Insn>> {
    fn default() -> Self {
        use crate::subroutine::ToSubroutine;
        use crate::Step;
        crate::Sequence::<Insn>::first().to_subroutine()
    }
}

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
        Self(s, std::marker::PhantomData)
    }
}

impl<Insn: ShouldReturn, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> BruteforceSearch<Insn>
    for Subroutine<Insn, S>
{
    fn analyze_this(&self) -> Result<(), crate::StaticAnalysis<Insn>>
    where
        Self: Sized,
    {
        let seq = self.0.as_ref();
        let offs = seq.len() - 1;
        if let Some(sa) = seq[offs].should_return() {
            return Err(sa.set_offset(offs));
        }
        Ok(())
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

impl<Insn: ShouldReturn, S: crate::Step + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    crate::Step for Subroutine<Insn, S>
{
    fn first() -> Self {
        let mut r = Self(S::first(), std::marker::PhantomData);
        r.step();
        r.fixup();
        r
    }
    fn next(&mut self) -> crate::IterationResult {
        self.0.next()?;
        self.fixup();
        Ok(())
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
