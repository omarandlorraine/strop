use crate::z80::Emulator;
use crate::z80::Insn;
use crate::CallingConvention;
use crate::Sequence;

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
#[derive(Debug)]
pub struct SdccCall1;

impl CallingConvention<Sequence<Insn>, u16, u16> for SdccCall1 {
    fn call(instructions: &Sequence<Insn>, input: u16) -> Option<u16> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(instructions);
        Some(emu.get())
    }
}