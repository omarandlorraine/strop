//! Module implementing AAPCS32, a Callable and Searchable complying with ARM's 32-bit calling
//! convention.
//!
//! This is good for searching for pure leaf AAPCS32 functions that take at least one argument and
//! return at least one thing.

use crate::RunResult;
use crate::backends::armv4t::Instruction;
use crate::test::{Parameters, ReturnValue};

const REGISTERS: [unarm::Reg; 13] = [
    unarm::Reg::R0,
    unarm::Reg::R1,
    unarm::Reg::R2,
    unarm::Reg::R3,
    unarm::Reg::R4,
    unarm::Reg::R5,
    unarm::Reg::R6,
    unarm::Reg::R7,
    unarm::Reg::R8,
    unarm::Reg::R9,
    unarm::Reg::R10,
    unarm::Reg::R11,
    unarm::Reg::R12,
];

/// Searches for functions complying to the Aapcs32 calling convention
#[derive(Clone)]
pub struct Aapcs32<Input: Parameters, Output: ReturnValue> {
    seq: crate::Sequence<Instruction>,
    args: std::cell::Cell<Option<usize>>,
    vals: std::cell::Cell<Option<usize>>,
    _phantom_data: std::marker::PhantomData<(Input, Output)>,
}

impl<Input: Parameters, Output: ReturnValue> From<crate::Sequence<Instruction>>
    for Aapcs32<Input, Output>
{
    fn from(seq: crate::Sequence<Instruction>) -> Self {
        Self {
            seq,
            args: Default::default(),
            vals: Default::default(),
            _phantom_data: Default::default(),
        }
    }
}

impl<Input: Parameters, Output: ReturnValue> Aapcs32<Input, Output> {
    /// returns fixups reducing the search space, such as skipping redundant encodings, or
    /// performing dataflow analysis
    fn reduce_search_space(&self) -> crate::StaticAnalysis<Instruction> {
        crate::dataflow::uninitialized(
            &self.seq,
            &crate::backends::armv4t::dataflow::ConditionFlags,
        )?;
        crate::dataflow::dont_expect_write(
            &self.seq,
            &crate::backends::armv4t::dataflow::ConditionFlags,
        )?;
        if let Some(arg) = self.args.get() {
            for a in &REGISTERS[..arg] {
                crate::dataflow::expect_read(&self.seq, a)?;
            }
            for a in &REGISTERS[arg..] {
                crate::dataflow::uninitialized(&self.seq, a)?;
            }
        }
        if let Some(val) = self.vals.get() {
            for a in &REGISTERS[..val] {
                crate::dataflow::expect_write(&self.seq, a)?;
            }
            for a in &REGISTERS[val..] {
                crate::dataflow::dont_expect_write(&self.seq, a)?;
            }
        }

        /*
                crate::dataflow::allocate_registers(&self.seq, &REGISTERS[..])?;
        */

        Ok(())
    }

    /// returns fixups ensuring compliance with the AAPCS32 calling convention.
    fn correctitudes(&self) -> crate::StaticAnalysis<Instruction> {
        self.seq.check_last(Instruction::make_bx_lr)?;
        crate::dataflow::leave_alone_except_last(&self.seq, &unarm::Reg::Lr)?;
        crate::dataflow::leave_alone_except_last(&self.seq, &unarm::Reg::Sp)?;
        crate::dataflow::leave_alone_except_last(&self.seq, &unarm::Reg::Pc)?;

        Ok(())
    }
    fn apply_all_fixups(&mut self) {
        self.make_correct();
        while let Err(fixup) = self.reduce_search_space() {
            self.seq.apply(&fixup);
            self.make_correct();
        }
    }
    fn make_correct(&mut self) {
        while let Err(fixup) = self.correctitudes() {
            self.seq.apply(&fixup);
        }
    }
}

impl<Input: Parameters, Output: ReturnValue> std::fmt::Display for Aapcs32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.seq)
    }
}

impl<Input: Parameters, Output: ReturnValue> std::fmt::Debug for Aapcs32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.seq)
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Callable<Input, Output>
    for Aapcs32<Input, Output>
{
    fn call(&self, parameters: Input) -> RunResult<Output> {
        let mut runner = Aapcs32Runner::default();
        parameters.put(&mut runner)?;
        self.args.set(Some(runner.arg));
        runner.call_subroutine(&self.seq)?;
        let result = Output::get(&mut runner);
        self.vals.set(Some(runner.val));
        result
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Traverse for Aapcs32<Input, Output> {
    fn increment(&mut self) {
        self.seq.increment();
        self.apply_all_fixups();
    }
    fn mutate(&mut self) {
        self.seq.mutate();
        self.make_correct();
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            seq: crate::Sequence::<Instruction>::from_bytes(bytes)?,
            _phantom_data: Default::default(),
            args: Default::default(),
            vals: Default::default(),
        })
    }
}

impl<Input: Parameters, Output: ReturnValue> Default for Aapcs32<Input, Output> {
    fn default() -> Self {
        Self {
            seq: crate::Sequence::<Instruction>::first(),
            _phantom_data: Default::default(),
            args: Default::default(),
            vals: Default::default(),
        }
    }
}

/// A basic emulator
#[derive(Default)]
pub struct Aapcs32Runner {
    /// The emulated system's memory
    pub mem: armv4t_emu::ExampleMem,
    /// The emulated system's CPU
    pub cpu: armv4t_emu::Cpu,

    pub arg: usize,
    pub val: usize,
}

impl crate::test::TakeParameters for Aapcs32Runner {
    fn put_bool(&mut self, v: bool) -> RunResult<()> {
        self.put_argument(v as u32)
    }
    fn put_i8(&mut self, v: i8) -> RunResult<()> {
        self.put_argument(v as i32 as u32)
    }
    fn put_u8(&mut self, v: u8) -> RunResult<()> {
        self.put_argument(v as u32)
    }
    fn put_i16(&mut self, v: i16) -> RunResult<()> {
        self.put_argument(v as i32 as u32)
    }
    fn put_u16(&mut self, v: u16) -> RunResult<()> {
        self.put_argument(v as u32)
    }
    fn put_i32(&mut self, v: i32) -> RunResult<()> {
        self.put_argument(v as u32)
    }
    fn put_u32(&mut self, v: u32) -> RunResult<()> {
        self.put_argument(v)
    }
    fn put_f32(&mut self, v: f32) -> RunResult<()> {
        self.put_argument(v.to_bits())
    }
}

impl crate::test::GetReturnValues for Aapcs32Runner {
    fn get_bool(&mut self) -> RunResult<bool> {
        Ok(self.get_value()? != 0)
    }
    fn get_i8(&mut self) -> RunResult<i8> {
        Ok(self.get_value()? as i8)
    }
    fn get_u8(&mut self) -> RunResult<u8> {
        Ok(self.get_value()? as u8)
    }
    fn get_i16(&mut self) -> RunResult<i16> {
        Ok(self.get_value()? as i16)
    }
    fn get_u16(&mut self) -> RunResult<u16> {
        Ok(self.get_value()? as u16)
    }
    fn get_u32(&mut self) -> RunResult<u32> {
        self.get_value()
    }
    fn get_i32(&mut self) -> RunResult<i32> {
        Ok(self.get_value()? as i32)
    }
    fn get_f32(&mut self) -> RunResult<f32> {
        Ok(self.get_value()? as f32)
    }
}

impl Aapcs32Runner {
    pub fn call_subroutine(&mut self, subroutine: &crate::Sequence<Instruction>) -> RunResult<()> {
        use crate::RunError;
        use armv4t_emu::Memory;
        use armv4t_emu::reg;

        // Write the subroutine to the beginning of the emulated CPU's address space
        for (address, instruction) in subroutine.iter().enumerate() {
            self.mem
                .w32((address << 2).try_into().unwrap(), instruction.0);
        }

        let mode = self.cpu.mode();
        const RETURN_ADDRESS: u32 = 0x5678;
        const BOTTOM_OF_STACK: u32 = 0x1000;

        self.cpu.reg_set(mode, reg::PC, 0);
        self.cpu.reg_set(mode, reg::SP, BOTTOM_OF_STACK);
        self.cpu.reg_set(mode, reg::LR, RETURN_ADDRESS);

        let end_of_subroutine = subroutine.to_bytes().len() as u32;

        for _ in 0..10000 {
            let pc = self.cpu.reg_get(mode, reg::PC);
            let sp = self.cpu.reg_get(mode, reg::SP);
            //dbg!(pc, Instruction(self.mem.r32(pc))); // for a basic execution trace

            if pc == RETURN_ADDRESS && sp == BOTTOM_OF_STACK {
                // Expected values for PC and SP mean that the subroutine has returned
                return Ok(());
            }
            if sp < BOTTOM_OF_STACK {
                // Stack underflow; this is not going to go well.
                return Err(RunError::RanAmok);
            }
            if pc >= end_of_subroutine {
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            self.cpu.step(&mut self.mem);
        }

        Err(crate::RunError::RanAmok)
    }
    fn put_argument(&mut self, arg: u32) -> RunResult<()> {
        let r = if (0..5).contains(&self.arg) {
            self.cpu
                .reg_set(armv4t_emu::Mode::User, self.arg as u8, arg);
            Ok(())
        } else {
            Err(crate::RunError::TooManyArguments)
        };
        self.arg += 1;
        r
    }
    fn get_value(&mut self) -> RunResult<u32> {
        let r = match self.val {
            0 => Ok(self.cpu.reg_get(armv4t_emu::Mode::User, 0)),
            1 => Ok(self.cpu.reg_get(armv4t_emu::Mode::User, 1)),
            _ => Err(crate::RunError::TooManyReturnValues),
        };
        self.val += 1;
        r
    }
}
