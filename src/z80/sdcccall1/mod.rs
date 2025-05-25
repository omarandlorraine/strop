use crate::test::Vals;
use crate::z80::dataflow::Register;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::BruteForce;
use crate::BruteforceSearch;
use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;

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
    fn fixup(seq: &Sequence<Insn>) -> Result<(), StaticAnalysis<Insn>>;
}

impl ReturnValue for u8 {
    fn get(emu: &Emulator) -> u8 {
        emu.get_a()
    }
    fn fixup(seq: &Sequence<Insn>) -> Result<(), StaticAnalysis<Insn>> {
        crate::dataflow::dont_expect_write(seq, &Register::B)
    }
}

impl ReturnValue for i8 {
    fn get(emu: &Emulator) -> i8 {
        emu.get_a() as i8
    }
    fn fixup(seq: &Sequence<Insn>) -> Result<(), StaticAnalysis<Insn>> {
        crate::dataflow::dont_expect_write(seq, &Register::B)
    }
}

impl ReturnValue for u16 {
    fn get(emu: &Emulator) -> u16 {
        emu.get_hl()
    }
    fn fixup(seq: &Sequence<Insn>) -> Result<(), StaticAnalysis<Insn>> {
        for reg in [Register::B, Register::C, Register::D, Register::E] {
            crate::dataflow::dont_expect_write(seq, &reg)?;
            crate::dataflow::uninitialized(seq, &reg)?;
        }
        for reg in [Register::H, Register::L] {
            crate::dataflow::uninitialized(seq, &reg)?;
        }
        Ok(())
    }
}

impl ReturnValue for i16 {
    fn get(emu: &Emulator) -> i16 {
        emu.get_hl() as i16
    }
    fn fixup(seq: &Sequence<Insn>) -> Result<(), StaticAnalysis<Insn>> {
        for reg in [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L] {
            crate::dataflow::dont_expect_write(seq, &reg)?;
            crate::dataflow::uninitialized(seq, &reg)?;
        }
        Ok(())
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct SdccCall1<Params, RetVal> {
    seq: crate::z80::Subroutine,
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
        use crate::Run;
        let mut emu = Emulator::default();
        input.put(&mut emu);
        self.seq.run(&mut emu)?;
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

impl<Params, RetVal: ReturnValue> BruteforceSearch<Insn> for SdccCall1<Params, RetVal> {
    fn analyze_this(&self) -> Result<(), StaticAnalysis<Insn>> {
        RetVal::fixup(self.seq.as_ref())?;
        crate::dataflow::uninitialized(self.seq.as_ref(), &dez80::register::Flag::C)?;
        crate::dataflow::uninitialized(self.seq.as_ref(), &dez80::register::Flag::Z)?;
        Ok(())
    }
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue, TargetFunction: Callable<Params, RetVal>>
    crate::AsBruteforce<Insn, Params, RetVal, TargetFunction> for SdccCall1<Params, RetVal>
{
    fn bruteforce(
        self,
        function: TargetFunction,
    ) -> BruteForce<Insn, Params, RetVal, TargetFunction, SdccCall1<Params, RetVal>> {
        BruteForce::<Insn, Params, RetVal, TargetFunction, SdccCall1<Params, RetVal>>::new(
            function, self,
        )
    }
}
