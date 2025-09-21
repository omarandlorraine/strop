use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::test::Input;
use crate::test::Output;
use crate::z80::Emulator;
use crate::z80::Insn;

pub trait ParameterList: Copy + Input {
    fn put(&self, emu: &mut Emulator);
}

impl ParameterList for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_a(*self);
    }
}

impl ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
    }
}

// TODO: Implement this for more types. The calling convention supports return signed types, 32-bit
// types, and perhaps others which are not supported (yet)
pub trait ReturnValue: Copy + Output + PartialEq {
    fn get(emu: &Emulator) -> Self;
}

impl ReturnValue for u8 {
    fn get(emu: &Emulator) -> u8 {
        emu.get_a()
    }
}

impl ReturnValue for i8 {
    fn get(emu: &Emulator) -> i8 {
        emu.get_a() as i8
    }
}

impl ReturnValue for u16 {
    fn get(emu: &Emulator) -> u16 {
        emu.get_hl()
    }
}

impl ReturnValue for i16 {
    fn get(emu: &Emulator) -> i16 {
        emu.get_hl() as i16
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct SdccCall1<Params, RetVal> {
    seq: Sequence<Insn>,
    params: std::marker::PhantomData<Params>,
    retval: std::marker::PhantomData<RetVal>,
}

impl<Params, RetVal> Default for SdccCall1<Params, RetVal> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Params, RetVal> SdccCall1<Params, RetVal> {
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        use crate::Step;
        Self::first()
    }

    // Performs static analysis on the code sequence
    fn analyze_this(&self) -> StaticAnalysis<Insn> {
        crate::subroutine::make_return(&self.seq)?;
        Ok(())
    }
}

impl<Params, RetVal> crate::Disassemble for SdccCall1<Params, RetVal> {
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for SdccCall1<Params, RetVal>
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        let mut emu = Emulator::init(&input);
        emu.call_subroutine(&self.seq)?;
        Ok(RetVal::get(&emu))
    }
}

impl<Params, RetVal> crate::Step for SdccCall1<Params, RetVal> {
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

impl<Params: ParameterList, RetVal: ReturnValue> crate::bruteforce::BruteForceSearch
    for SdccCall1<Params, RetVal>
{
    fn next(&mut self) -> crate::IterationResult {
        use crate::Step;

        self.seq.next()?;
        while let Err(sa) = self.analyze_this() {
            self.seq.apply_fixup(&sa);
        }

        Ok(())
    }
}
