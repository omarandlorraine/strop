//! An ad-hoc calling convention using zero-page for in and out.
//!
//! Idiomatic 6502 code often features subroutines that read parameters from zero-page, and write
//! output to zero-page. It will frequently use temporary variables in zeropage also. This module
//! implements a Searchable that formalizes that so strop can search for that kind of thing.

use crate::RunResult;
use crate::Sequence;
use crate::backends::mos6502::Instruction;
use crate::backends::mos6502::instruction_set::Datum;
use crate::test::{Parameters, ReturnValue};
use mos6502::Variant;

/// Searches for functions passing arguments and returning values in Zero Page
#[derive(Clone)]
pub struct ZpCall<V: Variant, I: Parameters, O: ReturnValue> {
    subroutine: Sequence<Instruction<V>>,
    args: std::cell::Cell<Option<usize>>,
    vals: std::cell::Cell<Option<usize>>,

    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> ZpCall<V, Input, Output> {
    #[rustfmt::skip]
    fn find_culls(&self) -> crate::StaticAnalysis<Instruction<V>> {
        use crate::cull;

        cull!(self.subroutine, Instruction::<V>::uses_absolute_mode_unnecessarily);
        cull!(self.subroutine, Instruction::<V>::no_operation);
        cull!(self.subroutine, Instruction::<V>::no_jams);
        cull!(self.subroutine, Instruction::<V>::no_pointers);
        cull!(self.subroutine, Instruction::<V>::no_flow_control);
        cull!(self.subroutine, |insn: &Instruction::<V>| insn.read_protect(64..=65535));
        cull!(self.subroutine, |insn: &Instruction::<V>| insn.write_protect(1..=63));
        cull!(self.subroutine, |insn: &Instruction::<V>| insn.write_protect(128..=65535));

        if let Some(u) = self.args.get() {
            let u = u as u16;
            cull!(self.subroutine, |insn: &Instruction::<V>| insn.read_protect(u..=65535));
        }

        crate::dataflow::uninitialized(&self.subroutine, &Datum::A)?;
        crate::dataflow::uninitialized(&self.subroutine, &Datum::X)?;
        crate::dataflow::uninitialized(&self.subroutine, &Datum::Y)?;

        Ok(())
    }
    fn make_correct(&mut self) {
        while let Err(e) = self.find_culls() {
            self.subroutine.apply(&e);
        }
    }
}
impl<V: Variant, Input: Parameters, Output: ReturnValue> crate::Traverse
    for ZpCall<V, Input, Output>
{
    fn increment(&mut self) {
        self.subroutine.increment();
        self.make_correct();
    }
    fn mutate(&mut self) {
        self.subroutine.mutate();
        self.make_correct();
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            subroutine: crate::Sequence::<Instruction<V>>::from_bytes(bytes)?,
            args: Default::default(),
            vals: Default::default(),
            _phantom: Default::default(),
        })
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> std::fmt::Display
    for ZpCall<V, Input, Output>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.subroutine)
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> std::fmt::Debug
    for ZpCall<V, Input, Output>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.subroutine)
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> Default for ZpCall<V, Input, Output> {
    fn default() -> Self {
        Self {
            subroutine: crate::Sequence::<Instruction<V>>::first(),
            args: Default::default(),
            vals: Default::default(),
            _phantom: Default::default(),
        }
    }
}

#[derive(Default)]
struct ZpRunner<V: Variant> {
    /// The emulator
    pub cpu: mos6502::cpu::CPU<mos6502::memory::Memory, V>,

    pub arg: usize,
    pub val: usize,
}

impl<V: Variant> ZpRunner<V> {
    fn call_subroutine(&mut self, subroutine: &Sequence<Instruction<V>>) -> RunResult<()> {
        use mos6502::memory::Bus;

        const START_OF_SUBROUTINE: u16 = 0x2000;
        let end_of_subroutine: u16 = START_OF_SUBROUTINE + subroutine.to_bytes().len() as u16;

        self.cpu
            .memory
            .set_bytes(START_OF_SUBROUTINE, &subroutine.to_bytes());
        self.cpu.registers.program_counter = START_OF_SUBROUTINE;
        for _ in 0..1000 {
            if !self.cpu.single_step() {
                // couldn't single step.
                // For example, the CPU ran into an undecodable opcode.
                // Or, for example on CMOS ones which can enter wait states,
                return Err(crate::RunError::RanAmok);
            }
            if !(START_OF_SUBROUTINE..=end_of_subroutine)
                .contains(&self.cpu.registers.program_counter)
            {
                return Err(crate::RunError::RanAmok);
            }
            if self.cpu.registers.program_counter == end_of_subroutine {
                return Ok(());
            }
        }
        Err(crate::RunError::RanAmok)
    }

    fn get_value(&mut self) -> RunResult<u8> {
        use mos6502::memory::Bus;
        if (0..64).contains(&self.arg) {
            let r = self
                .cpu
                .memory
                .get_byte((self.val + 64).try_into().unwrap());
            self.val += 1;
            Ok(r)
        } else {
            Err(crate::RunError::TooManyArguments)
        }
    }
}

impl<V: Variant> crate::test::GetReturnValues for ZpRunner<V> {
    fn get_bool(&mut self) -> RunResult<bool> {
        Ok(self.get_value()? != 0)
    }
    fn get_i8(&mut self) -> RunResult<i8> {
        Ok(self.get_value()? as i8)
    }
    fn get_u8(&mut self) -> RunResult<u8> {
        self.get_value()
    }
    fn get_i16(&mut self) -> RunResult<i16> {
        let a = self.get_value()?;
        let b = self.get_value()?;
        Ok(i16::from_le_bytes([a, b]))
    }
    fn get_u16(&mut self) -> RunResult<u16> {
        let a = self.get_value()?;
        let b = self.get_value()?;
        Ok(u16::from_le_bytes([a, b]))
    }
    fn get_u32(&mut self) -> RunResult<u32> {
        let a = self.get_value()?;
        let b = self.get_value()?;
        let c = self.get_value()?;
        let d = self.get_value()?;
        Ok(u32::from_le_bytes([a, b, c, d]))
    }
    fn get_i32(&mut self) -> RunResult<i32> {
        let a = self.get_value()?;
        let b = self.get_value()?;
        let c = self.get_value()?;
        let d = self.get_value()?;
        Ok(i32::from_le_bytes([a, b, c, d]))
    }
    fn get_f32(&mut self) -> RunResult<f32> {
        Err(crate::RunError::UnsupportedType)
    }
}

impl<V: Variant> crate::test::TakeParameters for ZpRunner<V> {
    fn put_bool(&mut self, v: bool) -> RunResult<()> {
        self.put_u8(if v { 0x80 } else { 0x00 })
    }
    fn put_i8(&mut self, v: i8) -> RunResult<()> {
        self.put_u8(v as u8)
    }
    fn put_u8(&mut self, v: u8) -> RunResult<()> {
        use mos6502::memory::Bus;

        let r = if (0..64).contains(&self.arg) {
            self.cpu.memory.set_bytes(self.arg as u16, &[v]);
            Ok(())
        } else {
            Err(crate::RunError::TooManyArguments)
        };
        self.arg += 1;
        r
    }
    fn put_i16(&mut self, v: i16) -> RunResult<()> {
        for b in v.to_le_bytes() {
            self.put_u8(b)?;
        }
        Ok(())
    }
    fn put_u16(&mut self, v: u16) -> RunResult<()> {
        for b in v.to_le_bytes() {
            self.put_u8(b)?;
        }
        Ok(())
    }
    fn put_i32(&mut self, v: i32) -> RunResult<()> {
        for b in v.to_le_bytes() {
            self.put_u8(b)?;
        }
        Ok(())
    }
    fn put_u32(&mut self, v: u32) -> RunResult<()> {
        for b in v.to_le_bytes() {
            self.put_u8(b)?;
        }
        Ok(())
    }
    fn put_f32(&mut self, _v: f32) -> RunResult<()> {
        Err(crate::RunError::UnsupportedType)
    }
}

impl<V: Variant + Default, Input: Parameters, Output: ReturnValue> crate::Callable<Input, Output>
    for ZpCall<V, Input, Output>
{
    fn call(&self, parameters: Input) -> RunResult<Output> {
        let mut runner = ZpRunner::default();
        parameters.put(&mut runner)?;
        self.args.set(Some(runner.arg));
        runner.call_subroutine(&self.subroutine)?;
        let result = Output::get(&mut runner);
        self.vals.set(Some(runner.val));
        result
    }
}
