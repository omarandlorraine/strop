//! Mimics the O32 calling convention

use crate::Callable;
use crate::Disassemble;
use crate::Sequence;
use crate::mips::Insn;
use crate::mips::emu::Parameters;
use crate::mips::emu::ReturnValue;
use crate::test::Vals;

/// Searches for functions complying to the O32 calling convention
#[derive(Clone, Debug, Default)]
pub struct O32<Params: Copy + Vals + Parameters, RetVal: Copy + Vals + ReturnValue> {
    seq: Sequence<Insn>,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<RetVal>,
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
    fn analyze(&self) -> crate::StaticAnalysis<Insn> {
        use trapezoid_core::cpu::RegisterType;
        crate::mips::optimizer::skip_pointless_instructions(&self.seq)?;
        crate::subroutine::leaf_subroutine(&self.seq)?;

        Params::analyze_this(&self.seq)?;
        RetVal::analyze_this(&self.seq)?;

        for reg in [RegisterType::Zero, RegisterType::At, RegisterType::Sp] {
            crate::dataflow::dont_expect_write(&self.seq, &reg)?;
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
            crate::dataflow::uninitialized(&self.seq, &reg)?;
        }

        crate::dataflow::allocate_registers(
            &self.seq,
            &[
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
            ],
        )?;

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
            crate::dataflow::leave_alone(&self.seq, &reg)?;
        }
        crate::dataflow::leave_alone_except_last(&self.seq, &RegisterType::Ra)?;
        Ok(())
    }
}
