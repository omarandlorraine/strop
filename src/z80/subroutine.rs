use crate::z80::Insn;
use crate::Goto;
use crate::StropError;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with a
/// calling convention, so that it can be called using strop's supplied `Callable` trait.
#[derive(Debug)]
pub struct Subroutine<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> {
    sequence: crate::Sequence<Insn>,
    d: std::marker::PhantomData<T>,
    e: std::marker::PhantomData<P>,
    f: std::marker::PhantomData<R>,
}

impl<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> Subroutine<P, R, T> {
    pub fn new() -> Self {
        use crate::Iterable;

        Self {
            sequence: crate::Sequence::<Insn>::first(),
            d: Default::default(),
            e: Default::default(),
            f: Default::default(),
        }
    }
}

impl<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> AsRef<crate::Sequence<Insn>> for Subroutine<P, R, T> {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        &self.sequence
    }
}


impl<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> Goto<Insn> for Subroutine<P, R, T> {
    fn goto(&mut self, i: &[Insn]) {
        self.sequence.goto(i);
    }
}

impl<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> crate::Callable<crate::Sequence<Insn>, T, P, R> for Subroutine<P, R, T> {
    fn call(&self, parameters: P) -> Result<R, StropError<crate::Sequence<Insn>>> {
        T::call(&self.sequence, parameters)
    }
}

pub trait IntoSubroutine<P, R, T: crate::CallingConvention<crate::Sequence<Insn>, P, R>> {
    fn into_subroutine(instructions: &[Insn]) -> Subroutine<P, R, T>;
}
