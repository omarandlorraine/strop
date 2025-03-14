use crate::test::Vals;
use crate::z80::Emulator;
use crate::BruteForce;
use crate::Callable;

pub trait ParameterList: Copy + Vals {
    fn put(&self, emu: &mut Emulator);
    fn reglist() -> Vec<crate::z80::dataflow::Register>;
}

impl ParameterList for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_a(*self);
    }
    fn reglist() -> Vec<crate::z80::dataflow::Register> {
        vec![crate::z80::dataflow::Register::A]
    }
}

impl ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
    }
    fn reglist() -> Vec<crate::z80::dataflow::Register> {
        vec![crate::z80::dataflow::Register::H, crate::z80::dataflow::Register::L]
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
pub struct SdccCall1 {
    seq: crate::z80::Subroutine,
}

impl Default for SdccCall1 {
    fn default() -> Self {
        Self::new()
    }
}

impl SdccCall1 {
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        use crate::Step;
        Self::first()
    }
}

impl crate::Disassemble for SdccCall1 {
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal> for SdccCall1 {
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        use crate::Run;
        let mut emu = Emulator::default();
        input.put(&mut emu);
        self.seq.run(&mut emu)?;
        Ok(RetVal::get(&emu))
    }

    fn dataflow_fixup(&mut self) {
        use crate::dataflow::DataFlow;
        for i in Params::reglist().iter() {
            self.seq.make_read(i).unwrap();
        }
    }
}

impl crate::Step for SdccCall1 {
    fn first() -> Self {
        Self {
            seq: crate::Step::first(),
        }
    }

    fn next(&mut self) -> crate::IterationResult {
        self.seq.next()
    }
}

impl<
        InputParameters: ParameterList,
        ReturnType: ReturnValue,
        TargetFunction: Callable<InputParameters, ReturnType>,
    > crate::AsBruteforce<InputParameters, ReturnType, TargetFunction> for SdccCall1
{
    fn bruteforce(
        self,
        function: TargetFunction,
    ) -> BruteForce<InputParameters, ReturnType, TargetFunction, SdccCall1> {
        BruteForce::<InputParameters, ReturnType, TargetFunction, SdccCall1>::new(function, self)
    }
}
