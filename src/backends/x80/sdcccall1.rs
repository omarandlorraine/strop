//! This module implements a callable, which may be bruteforce-searched, and which adheres to the
//! SDCC_CALL(1) calling convention
use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;

use crate::backends::x80::EmuInterface;
use crate::backends::x80::SdccCallable;
use crate::backends::x80::data::Datum;

const ALL_REGISTERS: [Datum;13] = [
    Datum::A,
    Datum::B,
    Datum::C,
    Datum::D,
    Datum::E,
    Datum::H,
    Datum::L,
    Datum::R,
    Datum::I,
    Datum::Ixh,
    Datum::Ixl,
    Datum::Iyh,
    Datum::Iyl,
];

/// A trait defining how arguments get pushed to an emulator for SDCC_CALL(1).
///
/// This varies between SM83 and Z80 because testing by the SDCC project showed that register
/// allocation works better that way (these two architectures have very irregular register files,
/// so register allocation is not easy!) See https://arxiv.org/abs/2112.01397 for details.
pub trait SdccCall1PushPop {
    /// return the registers that have been written to
    fn input_registers(&self) -> Vec<crate::backends::x80::data::Datum>;

    /// return the registers that have been read from
    fn output_registers(&self) -> Vec<crate::backends::x80::data::Datum>;
}

/// A type representing a subroutine mimicking the calling convention used by modern-day SDCC.
/// SDCC's internal documentation calls this `__sdcccall(1)`.
#[derive(Default)]
pub struct SdccCall1<Instruction: SdccCallable> {
    seq: Sequence<Instruction>,
    // TODO: instead of RefCell<Vec<Datume>> is would save us many heap allocations if instead we
    // could use Cell<some bitmap or other trivially copyable type>
    args: std::cell::RefCell<Vec<Datum>>,
    vals: std::cell::RefCell<Vec<Datum>>,
}

impl<Instruction: SdccCallable> std::fmt::Display for SdccCall1<Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.seq)
    }
}

impl<Instruction: SdccCallable> std::fmt::Debug for SdccCall1<Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.seq)
    }
}

impl<Instruction: SdccCallable> SdccCall1<Instruction> {
    fn analyze(&self) -> StaticAnalysis<Instruction> {
        self.seq.check_last(Instruction::make_return)?;
        for reg in self.args.borrow().iter() {
            crate::dataflow::expect_read(&self.seq, reg)?;
        }
        for reg in self.vals.borrow().iter() {
            crate::dataflow::expect_write(&self.seq, reg)?;
        }
        for reg in ALL_REGISTERS.iter()
            .filter(|datum| self.vals.borrow().iter().any(|d| d != *datum)) {
            crate::dataflow::dont_expect_write(&self.seq, reg)?;
        }
        for reg in ALL_REGISTERS.iter()
            .filter(|datum| self.args.borrow().iter().any(|d| d != *datum)) {
            crate::dataflow::uninitialized(&self.seq, reg)?;
        }
        self.seq.check_all(Instruction::make_pure)?;
        Ok(())
    }
    fn make_correct(&mut self) {
        while let Err(fixup) = self.analyze() {
            self.seq.apply(&fixup);
        }
    }
}

impl<Instruction: SdccCallable, Params: crate::test::Parameters, RetVal: crate::test::ReturnValue>
    Callable<Params, RetVal> for SdccCall1<Instruction>
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        let mut emu = Instruction::Runner::default();
        input.put(&mut emu)?;
        *self.args.borrow_mut() = emu.input_registers();
        emu.call(self.seq.to_bytes())?;
        let r = RetVal::get(&mut emu)?;
        *self.vals.borrow_mut() = emu.output_registers();
        Ok(r)
    }
}

impl<Instruction: SdccCallable> crate::Traverse for SdccCall1<Instruction> {
    fn increment(&mut self) {
        self.seq.increment();
        self.make_correct();
    }
    fn mutate(&mut self) {
        self.seq.mutate();
        self.make_correct();
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            seq: crate::Sequence::<Instruction>::from_bytes(bytes)?,
            args: Default::default(),
            vals: Default::default(),
        })
    }
}
