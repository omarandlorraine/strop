use crate::armv4t::Emulator;
use crate::armv4t::Insn;
/// Module containing conveniences for searching for functions. Basically mimics what I see LLVM
/// and whatever doing.
///
use crate::armv4t::Subroutine;
use crate::IterableSequence;
use crate::StropError;

trait Parameters {
    fn put(&self, emu: &mut Emulator);
}

impl Parameters for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_r0((*self).into());
    }
}

impl Parameters for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_r0((*self).into());
    }
}

// TODO: Implement this for more types. The calling convention supports return signed types, 32-bit
// types, and perhaps others which are not supported (yet)
trait GetReturnValue<T> {
    fn get(&self) -> T;
}

impl GetReturnValue<u8> for Emulator {
    fn get(&self) -> u8 {
        self.get_r0() as u8
    }
}

impl GetReturnValue<i8> for Emulator {
    fn get(&self) -> i8 {
        self.get_r0() as i8
    }
}

impl GetReturnValue<u16> for Emulator {
    fn get(&self) -> u16 {
        self.get_r0() as u16
    }
}

impl GetReturnValue<i16> for Emulator {
    fn get(&self) -> i16 {
        self.get_r0() as i16
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct Function(Subroutine);

impl crate::Disassemble for Function {
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl AsRef<crate::Sequence<Insn>> for Function {
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.0.as_ref()
    }
}

impl<InputParameters: Parameters, ReturnValue> crate::Callable<InputParameters, ReturnValue>
    for Function
where
    Emulator: GetReturnValue<ReturnValue>,
{
    fn call(&self, input: InputParameters) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(self.0.as_ref())?;
        Ok(emu.get())
    }
}

impl IterableSequence for Function {
    fn first() -> Self {
        Self(Subroutine::new())
    }

    fn stride_at(&mut self, offset: usize) -> bool {
        self.0.stride_at(offset)
    }

    fn step_at(&mut self, offset: usize) -> bool {
        self.0.step_at(offset)
    }
}

impl crate::Goto<Insn> for Function {
    fn goto(&mut self, t: &[Insn]) {
        self.0.goto(t);
    }
}
