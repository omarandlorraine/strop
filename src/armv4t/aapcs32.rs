//! Implements searches for functions complying with the AAPCS32 calling convention, as used by
//! modern (EABI) linux systems and others.

use crate::BruteforceSearch;
use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::armv4t::Emulator;
use crate::armv4t::Insn;

const MODE: armv4t_emu::Mode = armv4t_emu::Mode::User;

use crate::armv4t::data::Register;

trait FitsInRegister {
    fn put(&self, emu: &mut Emulator, pos: u8);
    fn get(emu: &Emulator, pos: u8) -> Self;
}

impl FitsInRegister for u32 {
    fn put(&self, emu: &mut Emulator, pos: u8) {
        emu.cpu.reg_set(MODE, pos, *self);
    }
    fn get(emu: &Emulator, pos: u8) -> Self {
        emu.cpu.reg_get(MODE, pos)
    }
}

impl FitsInRegister for i32 {
    fn put(&self, emu: &mut Emulator, pos: u8) {
        emu.cpu.reg_set(MODE, pos, *self as u32);
    }
    fn get(emu: &Emulator, pos: u8) -> Self {
        emu.cpu.reg_get(MODE, pos) as i32
    }
}

impl FitsInRegister for f32 {
    fn put(&self, emu: &mut Emulator, pos: u8) {
        emu.cpu.reg_set(MODE, pos, self.to_bits())
    }
    fn get(emu: &Emulator, pos: u8) -> Self {
        Self::from_bits(emu.cpu.reg_get(MODE, pos))
    }
}

/// Trait for any type which may be used and the functino parameters (i.e. a single scalar or tuple
/// of scalars, I think that's what AAPCS32 defines as permissible)
pub trait ParameterList {
    /// Puts the parameters into the expected place in the emulator (that is, the parameters are
    /// written to the register file in the expected way for the function call)
    fn put_list(&self, emu: &mut Emulator);
    /// Performs data analysis on the instruction sequence, making sure it does not read from a
    /// non-argument register, etc.
    fn analyze(seq: &Sequence<Insn>) -> crate::StaticAnalysis<Insn>;
}

impl<T> ParameterList for T
where
    T: FitsInRegister,
{
    fn put_list(&self, emu: &mut Emulator) {
        self.put(emu, 0);
    }

    fn analyze(seq: &Sequence<Insn>) -> crate::StaticAnalysis<Insn> {
        crate::dataflow::leave_alone(seq, &Register::R1)?;
        crate::dataflow::leave_alone(seq, &Register::R2)?;
        crate::dataflow::leave_alone(seq, &Register::R3)?;
        crate::dataflow::uninitialized(seq, &Register::R4)?;
        crate::dataflow::uninitialized(seq, &Register::R5)?;
        crate::dataflow::uninitialized(seq, &Register::R6)?;
        crate::dataflow::uninitialized(seq, &Register::R7)?;
        crate::dataflow::uninitialized(seq, &Register::R8)?;
        crate::dataflow::uninitialized(seq, &Register::R9)?;
        crate::dataflow::uninitialized(seq, &Register::R10)?;
        crate::dataflow::uninitialized(seq, &Register::R11)?;
        crate::dataflow::uninitialized(seq, &Register::R12)?;
        crate::dataflow::leave_alone(seq, &Register::Sp)
    }
}

/// Trait for any type which may be used as a function's return value
pub trait ReturnValue {
    /// Gets the return value from the emulator's register file
    fn get_list(emu: &Emulator) -> Self;
}

impl<T> ReturnValue for T
where
    T: FitsInRegister,
{
    fn get_list(emu: &Emulator) -> Self {
        T::get(emu, 0)
    }
}

/// The AAPCS32-compliant function
#[derive(Clone, Debug, Default)]
pub struct Function<Params, RetVal> {
    seq: crate::Sequence<Insn>,
    params: std::marker::PhantomData<Params>,
    retval: std::marker::PhantomData<RetVal>,
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for Function<Params, RetVal>
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        let mut emu = Emulator::init(&input);
        emu.call_subroutine(&self.seq)?;
        Ok(RetVal::get_list(&emu))
    }
}

impl<Params, RetVal> crate::Disassemble for Function<Params, RetVal> {
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params, RetVal> crate::Goto<Insn> for Function<Params, RetVal> {
    fn goto(&mut self, t: &[Insn]) {
        self.seq.goto(t);
    }
}

impl<Params: ParameterList, RetVal> BruteforceSearch<Insn> for Function<Params, RetVal> {
    fn analyze_this(&self) -> StaticAnalysis<Insn> {
        // TODO: dataflow analysis could go here.
        crate::dataflow::uninitialized(&self.seq, &crate::armv4t::data::ConditionFlags)?;
        crate::subroutine::leaf_subroutine(&self.seq)?;
        crate::subroutine::make_return(&self.seq)?;
        Params::analyze(&self.seq)?;
        Ok(())
    }
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}
