//! This module implements a callable, which may be bruteforce-searched, and which adheres to the
//! SDCC_CALL(1) calling convention
use crate::BruteForce;
use crate::BruteforceSearch;
use crate::Callable;
use crate::Encode;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::Step;
use crate::test::Vals;

use crate::x80::EmuInterface;
use crate::x80::X80;

// TODO: Implement ParameterList and ReturnValue for more types. The calling convention supports
// signed types, 32-bit types, and perhaps others which are not supported (yet)

/// A type implementing this represents a parameter list, (i.e. a function's arguments), and knows
/// how to copy itself into an emulator as part of a function call.
pub trait ParameterList: Copy + Vals {
    /// Put the value(s) into the expected location(s) in the emulator.
    fn put<E: EmuInterface>(&self, emu: &mut E);
}

impl ParameterList for u8 {
    fn put<E: EmuInterface>(&self, emu: &mut E) {
        emu.set_a(*self);
    }
}

impl ParameterList for u16 {
    fn put<E: EmuInterface>(&self, emu: &mut E) {
        emu.set_hl(*self);
    }
}

/// A type implementing this represents a return value, (i.e. anything a function can return under
/// the calling convention), and knows how to copy itself from an emulator at the end of a function
/// call.
pub trait ReturnValue: Copy + Vals + PartialEq {
    /// Get the value from the emulator
    fn get<E: EmuInterface>(emu: &E) -> Self;
}

impl ReturnValue for u8 {
    fn get<E: EmuInterface>(emu: &E) -> u8 {
        emu.get_a()
    }
}

impl ReturnValue for i8 {
    fn get<E: EmuInterface>(emu: &E) -> i8 {
        emu.get_a() as i8
    }
}

impl ReturnValue for u16 {
    fn get<E: EmuInterface>(emu: &E) -> u16 {
        emu.get_hl()
    }
}

impl ReturnValue for i16 {
    fn get<E: EmuInterface>(emu: &E) -> i16 {
        emu.get_hl() as i16
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct SdccCall1<Insn: X80, Params, RetVal> {
    seq: Sequence<Insn>,
    params: std::marker::PhantomData<Params>,
    retval: std::marker::PhantomData<RetVal>,
}

impl<Insn: X80, Params, RetVal> Default for SdccCall1<Insn, Params, RetVal> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Insn: X80, Params, RetVal> SdccCall1<Insn, Params, RetVal> {
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        Self::first()
    }
}

impl<Insn: X80 + crate::Disassemble, Params, RetVal> crate::Disassemble
    for SdccCall1<Insn, Params, RetVal>
{
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Insn: X80 + Encode<u8>, Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for SdccCall1<Insn, Params, RetVal>
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        let mut emu = Insn::Emulator::new();
        input.put(&mut emu);
        emu.call(self.seq.encode())?;
        Ok(RetVal::get(&emu))
    }
}

impl<Insn: X80 + Clone, Params, RetVal> crate::Step for SdccCall1<Insn, Params, RetVal> {
    fn first() -> Self {
        Self {
            seq: crate::Step::first(),
            retval: std::marker::PhantomData,
            params: std::marker::PhantomData,
        }
    }

    fn next(&mut self) -> crate::IterationResult {
        self.seq.next()
    }
}

impl<Insn: X80, Params, RetVal> BruteforceSearch<Insn> for SdccCall1<Insn, Params, RetVal> {
    fn analyze_this(&self) -> StaticAnalysis<Insn> {
        crate::subroutine::make_return(&self.seq)?;
        Ok(())
    }
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}

impl<
    Insn: X80 + Encode<u8>,
    Params: ParameterList,
    RetVal: ReturnValue,
    TargetFunction: Callable<Params, RetVal>,
> crate::AsBruteforce<Insn, Params, RetVal, TargetFunction> for SdccCall1<Insn, Params, RetVal>
{
    fn bruteforce(
        self,
        function: TargetFunction,
    ) -> BruteForce<Insn, Params, RetVal, TargetFunction, SdccCall1<Insn, Params, RetVal>> {
        BruteForce::<Insn, Params, RetVal, TargetFunction, SdccCall1<Insn, Params, RetVal>>::new(
            function, self,
        )
    }
}
