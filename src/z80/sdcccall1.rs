use crate::z80::dataflow::Register;
use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::DataFlow;
use crate::StropError;

pub trait SdccCall1ParameterList {
    fn put(&self, emu: &mut Emulator);
    fn live_in() -> Vec<Register>;
}

impl SdccCall1ParameterList for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_a(*self);
    }
    fn live_in() -> Vec<Register> {
        vec![Register::A]
    }
}

impl SdccCall1ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
    }
    fn live_in() -> Vec<Register> {
        vec![Register::H, Register::L]
    }
}

// TODO: Implement this for more types. The calling convention supports return signed types, 32-bit
// types, and perhaps others which are not supported (yet)
trait SdccCall1GetReturnValue<T> {
    fn get(&self) -> T;
}

impl SdccCall1GetReturnValue<u8> for Emulator {
    fn get(&self) -> u8 {
        self.get_a()
    }
}

impl SdccCall1GetReturnValue<i8> for Emulator {
    fn get(&self) -> i8 {
        self.get_a() as i8
    }
}

impl SdccCall1GetReturnValue<u16> for Emulator {
    fn get(&self) -> u16 {
        self.get_hl()
    }
}

impl SdccCall1GetReturnValue<i16> for Emulator {
    fn get(&self) -> i16 {
        self.get_hl() as i16
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct SdccCall1<Params, ReturnValue> {
    subroutine: Subroutine,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<ReturnValue>,
}

impl<Params, ReturnValue> crate::Disassemble for SdccCall1<Params, ReturnValue> {
    fn dasm(&self) {
        self.subroutine.build().dasm()
    }
}

impl<Params, ReturnValue> AsRef<crate::Sequence<Insn>> for SdccCall1<Params, ReturnValue> {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.subroutine.as_ref()
    }
}

impl<Params, ReturnValue> std::ops::Deref for SdccCall1<Params, ReturnValue> {
    type Target = Subroutine;

    fn deref(&self) -> &Self::Target {
        &self.subroutine
    }
}

impl<Params, ReturnValue> std::ops::DerefMut for SdccCall1<Params, ReturnValue> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subroutine
    }
}

impl<Params: SdccCall1ParameterList, ReturnValue> SdccCall1<Params, ReturnValue> {
    /// Performs dataflow analysis on the function
    pub fn dataflow_analysis(&mut self) {
        for f in Params::live_in() {
            self.subroutine.make_read(&f);
        }
    }
}

impl<Params: SdccCall1ParameterList, ReturnValue> crate::Callable<Params, ReturnValue>
    for SdccCall1<Params, ReturnValue>
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    fn call(&self, input: Params) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.subroutine.build())?;
        Ok(emu.get())
    }
}

impl<Params, ReturnValue> crate::Goto<Insn> for SdccCall1<Params, ReturnValue> {
    fn goto(&mut self, t: &[Insn]) {
        self.subroutine.goto(t);
    }
}

impl<Params, ReturnValue> crate::Iterable for SdccCall1<Params, ReturnValue> {
    fn first() -> Self {
        Self {
            subroutine: crate::Iterable::first(),
            params: Default::default(),
            return_value: Default::default(),
        }
    }

    fn step(&mut self) -> bool {
        self.subroutine.step()
    }
}
