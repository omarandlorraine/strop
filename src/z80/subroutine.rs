use crate::z80::Insn;
use crate::Goto;
use crate::StropError;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with a
/// calling convention, so that it can be called using strop's supplied `Callable` trait.
#[derive(Debug)]
pub struct Subroutine<
    InputParameters,
    ReturnValue,
    T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
> {
    sequence: crate::Sequence<Insn>,
    d: std::marker::PhantomData<T>,
    e: std::marker::PhantomData<InputParameters>,
    f: std::marker::PhantomData<ReturnValue>,
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > Default for Subroutine<InputParameters, ReturnValue, T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > Subroutine<InputParameters, ReturnValue, T>
{
    //! Build a `Subroutine`
    /// Build a `Subroutine`
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

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > AsRef<crate::Sequence<Insn>> for Subroutine<InputParameters, ReturnValue, T>
{
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        &self.sequence
    }
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > Goto<Insn> for Subroutine<InputParameters, ReturnValue, T>
{
    fn goto(&mut self, i: &[Insn]) {
        self.sequence.goto(i);
    }
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > crate::Callable<crate::Sequence<Insn>, T, InputParameters, ReturnValue>
    for Subroutine<InputParameters, ReturnValue, T>
{
    fn call(
        &self,
        parameters: InputParameters,
    ) -> Result<ReturnValue, StropError<crate::Sequence<Insn>>> {
        T::call(&self.sequence, parameters)
    }
}

pub trait IntoSubroutine<
    InputParameters,
    ReturnValue,
    T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
>
{
    //! Build a `Subroutine`
    /// Build a `Subroutine`
    fn into_subroutine(instructions: &[Insn]) -> Subroutine<InputParameters, ReturnValue, T>;
}
