use crate::test::Vals;
use crate::z80::dataflow::Register;
use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::Callable;
use crate::DataFlow;
use crate::StropError;

pub trait SdccCall1ParameterList: Copy + Vals {
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
pub trait SdccCall1GetReturnValue<T: Copy + Vals> {
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
pub struct SdccCall1<Params: Copy + Vals, ReturnValue: Copy + Vals> {
    subroutine: Subroutine,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<ReturnValue>,
    pure_function: bool,
    leaf_function: bool,
}

impl<Params: SdccCall1ParameterList, ReturnValue: Copy + Vals + PartialEq> Default
    for SdccCall1<Params, ReturnValue>
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Params: SdccCall1ParameterList, ReturnValue: Copy + Vals + PartialEq>
    SdccCall1<Params, ReturnValue>
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        use crate::Iterable;
        Self::first()
    }

    /// Instantiates a strop::BruteForce object that searches over functions complying with the
    /// sdcccall(1) ABI.
    pub fn bruteforce<C: Clone + Callable<Params, ReturnValue>>(
        self,
        target_function: C,
    ) -> crate::BruteForce<Params, ReturnValue, C, SdccCall1<Params, ReturnValue>, Insn> {
        crate::BruteForce::new(target_function, self)
    }

    /// Makes sure that the search space includes only pure functions (i.e., those that do not have
    /// side effects, and do not observe any state other than its parameters).
    pub fn pure(&mut self) -> Self {
        self.pure_function = true;
        self.clone()
    }

    /// Makes sure that the search space includes only leaf functions (i.e., those that do not call
    /// other functions)
    pub fn leaf(&mut self) -> Self {
        self.leaf_function = true;
        self.clone()
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> crate::Disassemble
    for SdccCall1<Params, ReturnValue>
{
    fn dasm(&self) {
        self.subroutine.dasm()
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> AsRef<crate::Sequence<Insn>>
    for SdccCall1<Params, ReturnValue>
{
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.subroutine.as_ref()
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> std::ops::Deref
    for SdccCall1<Params, ReturnValue>
{
    type Target = Subroutine;

    fn deref(&self) -> &Self::Target {
        &self.subroutine
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> std::ops::DerefMut
    for SdccCall1<Params, ReturnValue>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subroutine
    }
}

impl<Params: Copy + Vals + SdccCall1ParameterList, ReturnValue: Copy + Vals>
    SdccCall1<Params, ReturnValue>
{
    /// Performs dataflow analysis on the function
    pub fn dataflow_analysis(&mut self) {
        for f in Params::live_in() {
            self.subroutine.make_read(&f);
        }
    }
}

impl<Params: Copy + Vals + SdccCall1ParameterList, ReturnValue: Copy + Vals>
    Callable<Params, ReturnValue> for SdccCall1<Params, ReturnValue>
where
    Emulator: SdccCall1GetReturnValue<ReturnValue>,
{
    fn call(&self, input: Params) -> Result<ReturnValue, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.subroutine)?;
        Ok(emu.get())
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> crate::Goto<Insn>
    for SdccCall1<Params, ReturnValue>
{
    fn goto(&mut self, t: &[Insn]) {
        self.subroutine.goto(t);
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> crate::Iterable
    for SdccCall1<Params, ReturnValue>
{
    fn first() -> Self {
        Self {
            subroutine: crate::Iterable::first(),
            params: Default::default(),
            return_value: Default::default(),
            leaf_function: false,
            pure_function: false,
        }
    }

    fn step(&mut self) -> bool {
        self.subroutine.step()
    }
}

impl<Params: Copy + Vals, ReturnValue: Copy + Vals> crate::Constrain<Insn>
    for SdccCall1<Params, ReturnValue>
{
    fn fixup(&mut self) {
        self.subroutine.fixup();
    }

    fn report(&self, offset: usize) -> Vec<String> {
        self.subroutine.report(offset)
    }
}
