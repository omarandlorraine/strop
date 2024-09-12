use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::IterableSequence;
use crate::StropError;

trait SdccCall1ParameterList {
    fn put(&self, emu: &mut Emulator);
}

impl SdccCall1ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
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
pub struct SdccCall1(Subroutine);

impl crate::Disassemble for SdccCall1 {
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl AsRef<crate::Sequence<Insn>> for SdccCall1 {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.0.as_ref()
    }
}

impl<InputParameters: SdccCall1ParameterList, ReturnValue>
    crate::Callable<InputParameters, ReturnValue> for SdccCall1
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    fn call(&self, input: InputParameters) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(self.0.as_ref())?;
        Ok(emu.get())
    }
}

impl IterableSequence for SdccCall1 {
    fn first() -> Self {
        Self(Subroutine::new())
    }

    fn stride_at(&mut self, offset: usize) -> bool {
        self.0.stride_at(offset);
        true
    }

    fn step_at(&mut self, offset: usize) -> bool {
        self.0.step_at(offset);
        true
    }
}

impl crate::Goto<Insn> for SdccCall1 {
    fn goto(&mut self, t: &[Insn]) {
        self.0.goto(t);
    }
}
