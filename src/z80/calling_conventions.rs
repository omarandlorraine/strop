use crate::z80::dataflow::Fact;
use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::StropError;

trait SdccCall1ParameterList {
    fn put(&self, emu: &mut Emulator);
    fn facts() -> Vec<Fact>;
}

impl SdccCall1ParameterList for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_a(*self);
    }
    fn facts() -> Vec<Fact> {
        vec![Fact::A]
    }
}

impl SdccCall1ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
    }
    fn facts() -> Vec<Fact> {
        vec![Fact::H, Fact::L]
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
        self.0.build().dasm()
    }
}

impl AsRef<crate::Sequence<Insn>> for SdccCall1 {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.0.as_ref()
    }
}

impl std::ops::Deref for SdccCall1 {
    type Target = Subroutine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SdccCall1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<InputParameters: SdccCall1ParameterList, ReturnValue>
    crate::Callable<InputParameters, ReturnValue> for SdccCall1
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    fn fixup(&mut self) {
        for f in InputParameters::facts() {
            crate::z80::dataflow::make_produce(&mut self.0, 0, f);
        }
    }

    fn call(&self, input: InputParameters) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.0.build())?;
        Ok(emu.get())
    }
}

impl crate::Goto<Insn> for SdccCall1 {
    fn goto(&mut self, t: &[Insn]) {
        self.0.goto(t);
    }
}

impl crate::Iterable for SdccCall1 {
    fn first() -> Self {
        Self(crate::Iterable::first())
    }

    fn step(&mut self) -> bool {
        self.0.step()
    }
}
