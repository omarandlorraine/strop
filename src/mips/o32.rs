//! Mimics the O32 calling convention

use crate::mips::emu::Parameters;
use crate::mips::emu::ReturnValue;
use crate::mips::Insn;
use crate::test::Vals;
use crate::Callable;
use crate::Disassemble;
use crate::Step;

/// Searches for functions complying to the O32 calling convention
#[derive(Clone, Debug)]
pub struct O32<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> {
    seq: crate::mips::Subroutine,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<RetVal>,
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> Default
    for O32<Params, RetVal>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> Step
    for O32<Params, RetVal>
{
    fn first() -> Self {
        Self {
            seq: Default::default(),
            params: Default::default(),
            return_value: Default::default(),
        }
    }

    fn next(&mut self) -> crate::IterationResult {
        self.seq.next()
    }
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> Callable<Params, RetVal>
    for O32<Params, RetVal>
{
    fn call(&self, p: Params) -> Result<RetVal, crate::RunError> {
        Ok(crate::mips::emu::call(&self.seq, p)?)
    }
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> Disassemble
    for O32<Params, RetVal>
{
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> O32<Params, RetVal> {
    /// Instantiates a new, empty O32.
    pub fn new() -> Self {
        use crate::Step;
        Self::first()
    }
}

impl<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue>
    crate::BruteforceSearch<Insn> for O32<Params, RetVal>
{
    fn analyze_this(&self) -> Option<crate::StaticAnalysis<Insn>> {
        // TODO: dataflow analysis could go here.
        None
    }
    fn inner(&mut self) -> &mut dyn crate::BruteforceSearch<Insn> {
        &mut self.seq
    }
}

impl<
        Params: Vals + Parameters,
        RetVal: ReturnValue + Vals + Clone,
        TargetFunction: Callable<Params, RetVal>,
    > crate::AsBruteforce<Insn, Params, RetVal, TargetFunction> for O32<Params, RetVal>
{
    fn bruteforce(
        self,
        function: TargetFunction,
    ) -> crate::BruteForce<Insn, Params, RetVal, TargetFunction, O32<Params, RetVal>> {
        crate::BruteForce::<Insn, Params, RetVal, TargetFunction, O32<Params, RetVal>>::new(
            function, self,
        )
    }
}
