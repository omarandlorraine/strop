use crate::z80::Insn;
use crate::Goto;
use crate::IterableSequence;
use crate::StropError;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with a
/// calling convention, so that it can be called using strop's supplied `Callable` trait.
#[derive(Clone, Debug)]
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
    > Subroutine<InputParameters, ReturnValue, T>
{
    fn fixup(&mut self) {
        use crate::Encode;
        while self.sequence[self.sequence.last_instruction_offset()].encode()[0] != 0xc9 {
            // make sure the subroutine ends in a return instruction
            self.sequence
                .stride_at(self.sequence.last_instruction_offset());
        }
    }
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > IterableSequence for Subroutine<InputParameters, ReturnValue, T>
{
    fn first() -> Self {
        Self::new()
    }

    fn stride_at(&mut self, offset: usize) -> bool {
        self.sequence.stride_at(offset);
        self.fixup();
        true
    }

    fn step_at(&mut self, offset: usize) -> bool {
        self.sequence.step_at(offset);
        self.fixup();
        true
    }
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
        use crate::IterableSequence;

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
    > crate::Disassemble for Subroutine<InputParameters, ReturnValue, T>
{
    fn dasm(&self) {
        self.sequence.dasm();
    }
}

impl<
        InputParameters,
        ReturnValue,
        T: crate::CallingConvention<crate::Sequence<Insn>, InputParameters, ReturnValue>,
    > crate::Callable<InputParameters, ReturnValue>
    for Subroutine<InputParameters, ReturnValue, T>
{
    fn call(&self, parameters: InputParameters) -> Result<ReturnValue, StropError> {
        use crate::Encode;
        let offs = self.sequence.last_instruction_offset();
        let last_instruction = self.sequence[offs];
        if last_instruction.encode()[0] != 0xc9 {
            // The subroutine doesn't end in a `RET` instruction; lunge the instruction at that
            // offset
            Err(StropError::Stride(offs))
        } else {
            T::call(&self.sequence, parameters)
        }
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
