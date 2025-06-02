//! Mimics the O32 calling convention

use crate::mips::emu::Parameters;
use crate::mips::emu::ReturnValue;
use crate::mips::Insn;
use crate::test::Vals;
use crate::Callable;
use crate::Disassemble;
use crate::Sequence;
use crate::Step;

/// Searches for functions complying to the O32 calling convention
#[derive(Clone, Debug)]
pub struct O32<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> {
    seq: Sequence<Insn>,
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
            seq: Step::first(),
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
        crate::mips::emu::call(&self.seq, p)
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
    fn analyze_this(&self) -> Result<(), crate::StaticAnalysis<Insn>> {
        use trapezoid_core::cpu::RegisterType;
        crate::mips::optimizer::skip_pointless_instructions(self.seq.as_ref())?;
        crate::subroutine::std_subroutine(&self.seq)?;

        Params::analyze_this(self.seq.as_ref())?;
        RetVal::analyze_this(self.seq.as_ref())?;

        for reg in [RegisterType::Zero, RegisterType::At, RegisterType::Sp] {
            crate::dataflow::dont_expect_write(self.seq.as_ref(), &reg)?;
        }

        for reg in [
            RegisterType::V0,
            RegisterType::V1,
            RegisterType::T0,
            RegisterType::T1,
            RegisterType::T2,
            RegisterType::T3,
            RegisterType::T4,
            RegisterType::T5,
            RegisterType::T6,
            RegisterType::T7,
            RegisterType::T8,
            RegisterType::T9,
        ] {
            crate::dataflow::uninitialized(self.seq.as_ref(), &reg)?;
        }

        for reg in [
            RegisterType::At,
            RegisterType::S0,
            RegisterType::S1,
            RegisterType::S2,
            RegisterType::S3,
            RegisterType::S4,
            RegisterType::S5,
            RegisterType::S6,
            RegisterType::S7,
            RegisterType::K0,
            RegisterType::K1,
            RegisterType::Gp,
            RegisterType::Sp,
            RegisterType::Fp,
        ] {
            crate::dataflow::leave_alone(self.seq.as_ref(), &reg)?;
        }
        crate::dataflow::leave_alone_except_last(self.seq.as_ref(), &RegisterType::Ra)?;
        Ok(())
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
