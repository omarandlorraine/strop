use crate::test::Vals;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::Callable;
use crate::Sequence;
use crate::StropError;

pub trait ParameterList: Copy + Vals {
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
pub trait ReturnValue: Copy + Vals + PartialEq {
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
pub struct SdccCall1<Params: Copy + Vals, RetVal: Copy + Vals> {
    seq: Sequence<Insn>,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<RetVal>,
}

impl<Params: ParameterList, RetVal: ReturnValue> Default for SdccCall1<Params, RetVal> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> SdccCall1<Params, RetVal> {
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        use crate::Step;
        Self::first()
    }

    /// Instantiates a strop::BruteForce object that searches over functions complying with the
    /// sdcccall(1) ABI.
    pub fn bruteforce<C: Clone + Callable<Params, RetVal>>(
        self,
        target_function: C,
    ) -> crate::BruteForce<Params, RetVal, C, SdccCall1<Params, RetVal>, Insn> {
        crate::BruteForce::new(target_function, self)
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Disassemble for SdccCall1<Params, RetVal> {
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> AsRef<crate::Sequence<Insn>>
    for SdccCall1<Params, RetVal>
{
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        &self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::Deref for SdccCall1<Params, RetVal> {
    type Target = Sequence<Insn>;

    fn deref(&self) -> &Self::Target {
        &self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::DerefMut for SdccCall1<Params, RetVal> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for SdccCall1<Params, RetVal>
{
    fn call(&self, input: Params) -> Result<RetVal, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.seq)?;
        Ok(RetVal::get(&emu))
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Goto<Insn> for SdccCall1<Params, RetVal> {
    fn goto(&mut self, t: &[Insn]) {
        self.seq.goto(t);
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Step for SdccCall1<Params, RetVal> {
    fn first() -> Self {
        Self {
            seq: crate::Step::first(),
            params: Default::default(),
            return_value: Default::default(),
        }
    }

    fn next(&mut self) -> crate::IterationResult {
        self.seq.next()
    }
}
