//! Implements searches for functions complying with the AAPCS32 calling convention, as used by
//! modern (EABI) linux systems and others.

use crate::armv4t::Emulator;
use crate::armv4t::Insn;
use crate::BruteforceSearch;
use crate::Callable;
use crate::StaticAnalysis;

const MODE: armv4t_emu::Mode = armv4t_emu::Mode::User;

/*
fn callee_saved(r: &Register) -> bool {
    match r {
        Register::R0 => false,
        Register::R1 => false,
        Register::R2 => false,
        Register::R3 => false,
        Register::R4 => true,
        Register::R5 => true,
        Register::R6 => true,
        Register::R7 => true,
        Register::R8 => true,
        Register::R9 => true,
        Register::R10 => true,
        Register::R11 => true,
        Register::R12 => false,
        Register::Sp => false,
        Register::Lr => false,
        Register::Pc => false,
    }
}
*/

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
}

impl<T> ParameterList for T
where
    T: FitsInRegister,
{
    fn put_list(&self, emu: &mut Emulator) {
        self.put(emu, 0);
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
    seq: crate::armv4t::Subroutine,
    params: std::marker::PhantomData<Params>,
    retval: std::marker::PhantomData<RetVal>,
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for Function<Params, RetVal>
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        use crate::Run;
        let mut emu = Emulator::default();
        input.put_list(&mut emu);
        self.seq.run(&mut emu)?;
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

impl<Params, RetVal> BruteforceSearch<Insn> for Function<Params, RetVal> {
    fn analyze_this(&self) -> Result<(), StaticAnalysis<Insn>> {
        // TODO: dataflow analysis could go here.
        Ok(())
    }
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}
