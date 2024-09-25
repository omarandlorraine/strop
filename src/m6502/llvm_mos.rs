use crate::m6502::subroutine::Subroutine;
use crate::m6502::Emulator;
use crate::m6502::Insn;
use crate::IterableSequence;
use crate::StropError;

trait LlvmMosParameterList<V: mos6502::Variant> {
    fn put(&self, emu: &mut Emulator<V>);
}

impl<V: mos6502::Variant> LlvmMosParameterList<V> for u8 {
    fn put(&self, emu: &mut Emulator<V>) {
        emu.set_a(*self);
    }
}

impl<V: mos6502::Variant> LlvmMosParameterList<V> for u16 {
    fn put(&self, emu: &mut Emulator<V>) {
        emu.set_ax(*self);
    }
}

// TODO: Implement this for more types. The calling convention supports return signed types, 32-bit
// types, and perhaps others which are not supported (yet)
trait SdccCall1GetReturnValue<T> {
    fn get(&self) -> T;
}

impl<V: mos6502::Variant> SdccCall1GetReturnValue<u8> for Emulator<V> {
    fn get(&self) -> u8 {
        self.get_a()
    }
}

impl<V: mos6502::Variant> SdccCall1GetReturnValue<i8> for Emulator<V> {
    fn get(&self) -> i8 {
        self.get_a() as i8
    }
}

impl<V: mos6502::Variant> SdccCall1GetReturnValue<u16> for Emulator<V> {
    fn get(&self) -> u16 {
        self.get_ax()
    }
}

impl<V: mos6502::Variant> SdccCall1GetReturnValue<i16> for Emulator<V> {
    fn get(&self) -> i16 {
        self.get_ax() as i16
    }
}

/// Mimics the calling convention used by llvm-mos.
#[derive(Clone, Debug)]
pub struct LlvmMos<V: mos6502::Variant + Clone>(Subroutine<V>);

impl<V: mos6502::Variant + Clone> crate::Disassemble for LlvmMos<V>
where
    Insn<V>: crate::Disassemble,
{
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl<V: mos6502::Variant + Clone> AsRef<crate::Sequence<Insn<V>>> for LlvmMos<V> {
    fn as_ref(&self) -> &crate::Sequence<Insn<V>> {
        self.0.as_ref()
    }
}

impl<
        V: mos6502::Variant + Clone + Default,
        InputParameters: LlvmMosParameterList<V>,
        ReturnValue,
    > crate::Callable<InputParameters, ReturnValue> for LlvmMos<V>
where
    Emulator<V>: SdccCall1GetReturnValue<ReturnValue>,
{
    fn call(&self, input: InputParameters) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::<V>::default();
        input.put(&mut emu);
        emu.run(self.0.as_ref())?;
        Ok(emu.get())
    }
}

impl<V: mos6502::Variant + Clone> IterableSequence for LlvmMos<V> {
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

impl<V: mos6502::Variant + Clone> crate::Goto<Insn<V>> for LlvmMos<V> {
    fn goto(&mut self, t: &[Insn<V>]) {
        self.0.goto(t);
    }
}
