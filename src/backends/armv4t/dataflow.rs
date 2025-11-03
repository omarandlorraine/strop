//! Dataflow analysis for ARM instructions.
//!
//! This module implements dataflow analysis for ARM instructions.
use crate::backends::armv4t::Instruction;
use crate::static_analysis::Fixup;

pub use unarm::Reg;

/// Condition flags. Conditional instructions read from this, and instructions which set the
/// condition flags (such as `ands`) write to the condition flags.
#[derive(Debug)]
pub struct ConditionFlags;

impl crate::dataflow::DataFlow<ConditionFlags> for Instruction {
    fn reads(&self, _datum: &ConditionFlags) -> bool {
        // reading from the Condition Flags means, the instruction is conditional.
        self.0 < 0xe000_0000
    }

    fn writes(&self, _datum: &ConditionFlags) -> bool {
        use unarm::Ins;
        match self.decode() {
            Ins::Adc { s, .. }
            | Ins::Add { s, .. }
            | Ins::And { s, .. }
            | Ins::Asr { s, .. }
            | Ins::Bic { s, .. }
            | Ins::Eor { s, .. }
            | Ins::Lsl { s, .. }
            | Ins::Lsr { s, .. }
            | Ins::Mla { s, .. }
            | Ins::Mov { s, .. }
            | Ins::Mul { s, .. }
            | Ins::Mvn { s, .. }
            | Ins::Orr { s, .. }
            | Ins::Ror { s, .. }
            | Ins::Rrx { s, .. }
            | Ins::Rsb { s, .. }
            | Ins::Rsc { s, .. }
            | Ins::Sbc { s, .. }
            | Ins::Sub { s, .. }
            | Ins::Umlal { s, .. }
            | Ins::Umull { s, .. } => s,

            Ins::Cmn { .. } | Ins::Cmp { .. } | Ins::Tst { .. } => true,

            Ins::B { .. }
            | Ins::Bkpt { .. }
            | Ins::Bl { .. }
            | Ins::Blx { .. }
            | Ins::Bx { .. }
            | Ins::Bxj { .. }
            | Ins::Cdp { .. }
            | Ins::Cdp2 { .. }
            | Ins::Clrex { .. }
            | Ins::Clz { .. }
            | Ins::Cps { .. }
            | Ins::Csdb { .. }
            | Ins::Dbg { .. }
            | Ins::Ldc { .. }
            | Ins::Ldc2 { .. }
            | Ins::Ldm { .. }
            | Ins::Ldr { .. }
            | Ins::Ldrb { .. }
            | Ins::Ldrbt { .. }
            | Ins::Ldrd { .. }
            | Ins::Ldrex { .. }
            | Ins::Ldrexb { .. }
            | Ins::Ldrexd { .. }
            | Ins::Ldrexh { .. }
            | Ins::Ldrh { .. }
            | Ins::Ldrsb { .. }
            | Ins::Ldrsh { .. }
            | Ins::Ldrt { .. }
            | Ins::Mcr { .. }
            | Ins::Mcr2 { .. }
            | Ins::Mcrr { .. }
            | Ins::Mcrr2 { .. }
            | Ins::Mrc { .. }
            | Ins::Mrc2 { .. }
            | Ins::Mrrc { .. }
            | Ins::Mrrc2 { .. }
            | Ins::Mrs { .. }
            | Ins::Msr { .. }
            | Ins::Neg { .. }
            | Ins::Nop { .. }
            | Ins::Pkhbt { .. }
            | Ins::Pkhtb { .. }
            | Ins::Pld { .. }
            | Ins::Pop { .. }
            | Ins::Push { .. }
            | Ins::Qadd { .. }
            | Ins::Qadd16 { .. }
            | Ins::Qadd8 { .. }
            | Ins::Qasx { .. }
            | Ins::Qdadd { .. }
            | Ins::Qdsub { .. }
            | Ins::Qsax { .. }
            | Ins::Qsub { .. }
            | Ins::Qsub16 { .. }
            | Ins::Qsub8 { .. }
            | Ins::Rev { .. }
            | Ins::Rev16 { .. }
            | Ins::Revsh { .. }
            | Ins::Rfe { .. }
            | Ins::Sadd16 { .. }
            | Ins::Sadd8 { .. }
            | Ins::Sasx { .. }
            | Ins::Sel { .. }
            | Ins::Setend { .. }
            | Ins::Sev { .. }
            | Ins::Shadd16 { .. }
            | Ins::Shadd8 { .. }
            | Ins::Shasx { .. }
            | Ins::Shsax { .. }
            | Ins::Shsub16 { .. }
            | Ins::Shsub8 { .. }
            | Ins::Smla { .. }
            | Ins::Smlad { .. }
            | Ins::Smlal { .. }
            | Ins::SmlalHalf { .. }
            | Ins::Smlald { .. }
            | Ins::Smlaw { .. }
            | Ins::Smlsd { .. }
            | Ins::Smlsld { .. }
            | Ins::Smmla { .. }
            | Ins::Smmls { .. }
            | Ins::Smmul { .. }
            | Ins::Smuad { .. }
            | Ins::Smul { .. }
            | Ins::Smull { .. }
            | Ins::Smulw { .. }
            | Ins::Smusd { .. }
            | Ins::Srs { .. }
            | Ins::Ssat { .. }
            | Ins::Ssat16 { .. }
            | Ins::Ssax { .. }
            | Ins::Ssub16 { .. }
            | Ins::Ssub8 { .. }
            | Ins::Stc { .. }
            | Ins::Stc2 { .. }
            | Ins::Stm { .. }
            | Ins::Str { .. }
            | Ins::Strb { .. }
            | Ins::Strbt { .. }
            | Ins::Strd { .. }
            | Ins::Strex { .. }
            | Ins::Strexb { .. }
            | Ins::Strexd { .. }
            | Ins::Strexh { .. }
            | Ins::Strh { .. }
            | Ins::Strt { .. }
            | Ins::Svc { .. }
            | Ins::Swp { .. }
            | Ins::Swpb { .. }
            | Ins::Sxtab { .. }
            | Ins::Sxtab16 { .. }
            | Ins::Sxtah { .. }
            | Ins::Sxtb { .. }
            | Ins::Sxtb16 { .. }
            | Ins::Sxth { .. }
            | Ins::Teq { .. }
            | Ins::Uadd16 { .. }
            | Ins::Uadd8 { .. }
            | Ins::Uasx { .. }
            | Ins::Udf { .. }
            | Ins::Uhadd16 { .. }
            | Ins::Uhadd8 { .. }
            | Ins::Uhasx { .. }
            | Ins::Uhsax { .. }
            | Ins::Uhsub16 { .. }
            | Ins::Uhsub8 { .. }
            | Ins::Umaal { .. }
            | Ins::Uqadd16 { .. }
            | Ins::Uqadd8 { .. }
            | Ins::Uqasx { .. }
            | Ins::Uqsax { .. }
            | Ins::Uqsub16 { .. }
            | Ins::Uqsub8 { .. }
            | Ins::Usad8 { .. }
            | Ins::Usada8 { .. }
            | Ins::Usat { .. }
            | Ins::Usat16 { .. }
            | Ins::Usax { .. }
            | Ins::Usub16 { .. }
            | Ins::Usub8 { .. }
            | Ins::Uxtab { .. }
            | Ins::Uxtab16 { .. }
            | Ins::Uxtah { .. }
            | Ins::Uxtb { .. }
            | Ins::Uxtb16 { .. }
            | Ins::Uxth { .. }
            | Ins::VabsF32 { .. }
            | Ins::VabsF64 { .. }
            | Ins::VaddF32 { .. }
            | Ins::VaddF64 { .. }
            | Ins::VcmpF32 { .. }
            | Ins::VcmpF64 { .. }
            | Ins::VcvtF32F64 { .. }
            | Ins::VcvtF32S32 { .. }
            | Ins::VcvtF32U32 { .. }
            | Ins::VcvtF64F32 { .. }
            | Ins::VcvtF64S32 { .. }
            | Ins::VcvtF64U32 { .. }
            | Ins::VcvtS32F32 { .. }
            | Ins::VcvtS32F64 { .. }
            | Ins::VcvtU32F32 { .. }
            | Ins::VcvtU32F64 { .. }
            | Ins::VdivF32 { .. }
            | Ins::VdivF64 { .. }
            | Ins::VldmF32 { .. }
            | Ins::VldmF64 { .. }
            | Ins::VldrF32 { .. }
            | Ins::VldrF64 { .. }
            | Ins::VmlaF32 { .. }
            | Ins::VmlaF64 { .. }
            | Ins::VmlsF32 { .. }
            | Ins::VmlsF64 { .. }
            | Ins::Vmov32Reg { .. }
            | Ins::VmovF32 { .. }
            | Ins::VmovF32Reg { .. }
            | Ins::VmovF64 { .. }
            | Ins::VmovReg32 { .. }
            | Ins::VmovRegF32 { .. }
            | Ins::VmovRegF32Dual { .. }
            | Ins::VmovF32RegDual { .. }
            | Ins::VmovRegF64 { .. }
            | Ins::VmovF64Reg { .. }
            | Ins::Vmrs { .. }
            | Ins::Vmsr { .. }
            | Ins::VmulF32 { .. }
            | Ins::VmulF64 { .. }
            | Ins::VnegF32 { .. }
            | Ins::VnegF64 { .. }
            | Ins::VnmlaF32 { .. }
            | Ins::VnmlaF64 { .. }
            | Ins::VnmlsF32 { .. }
            | Ins::VnmlsF64 { .. }
            | Ins::VnmulF32 { .. }
            | Ins::VnmulF64 { .. }
            | Ins::VpopF32 { .. }
            | Ins::VpopF64 { .. }
            | Ins::VpushF32 { .. }
            | Ins::VpushF64 { .. }
            | Ins::VsqrtF32 { .. }
            | Ins::VsqrtF64 { .. }
            | Ins::VstmF32 { .. }
            | Ins::VstmF64 { .. }
            | Ins::VstrF32 { .. }
            | Ins::VstrF64 { .. }
            | Ins::VsubF32 { .. }
            | Ins::VsubF64 { .. }
            | Ins::Wfe { .. }
            | Ins::Wfi { .. }
            | Ins::Yield { .. }
            | Ins::Word(..)
            | Ins::HalfWord(..)
            | Ins::Byte(..)
            | Ins::Illegal => false,
        }
    }

    fn sa(&self, offset: usize) -> Fixup<Self> {
        use crate::Instruction;
        Fixup::new("ConditionDataflow", Self::increment, offset)
    }
}

impl crate::dataflow::DataFlow<Reg> for Instruction {
    fn reads(&self, reg: &Reg) -> bool {
        use unarm::DefUseArgument;
        self.decode()
            .uses()
            .iter()
            .any(|r| *r == DefUseArgument::Reg(*reg))
    }

    fn writes(&self, reg: &Reg) -> bool {
        use unarm::DefUseArgument;
        self.decode()
            .defs()
            .iter()
            .any(|r| *r == DefUseArgument::Reg(*reg))
    }

    fn sa(&self, offset: usize) -> Fixup<Self> {
        use crate::Instruction;
        Fixup::new("RegisterDataflow", Self::increment, offset)
    }
}
