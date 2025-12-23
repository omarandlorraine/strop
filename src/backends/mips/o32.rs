//! Module implementing O32, a Callable and Searchable complying with MIPS's O32 calling
//! convention.
//!
//! This is good for searching for pure leaf O32 functions that take at least one argument and
//! return at least one thing.

use crate::RunResult;
use crate::backends::mips::Instruction;
use crate::test::{Parameters, ReturnValue};

/// Searches for functions complying to the O32 calling convention
#[derive(Clone)]
pub struct O32<Input: Parameters, Output: ReturnValue> {
    seq: crate::Sequence<Instruction>,
    args: std::cell::Cell<Option<usize>>,
    vals: std::cell::Cell<Option<usize>>,
    _phantom_data: std::marker::PhantomData<(Input, Output)>,
}

impl<Input: Parameters, Output: ReturnValue> O32<Input, Output> {
    /// returns fixups reducing the search space, such as skipping redundant encodings, or
    /// performing dataflow analysis
    fn reduce_search_space(&self) -> crate::StaticAnalysis<Instruction> {
        use trapezoid_core::cpu::RegisterType;
        self.seq.check_all(crate::Instruction::pointless)?;
        crate::dataflow::allocate_registers(
            &self.seq,
            &[
                RegisterType::T0,
                RegisterType::T1,
                RegisterType::T2,
                RegisterType::T3,
                RegisterType::T4,
                RegisterType::T5,
                RegisterType::T6,
                RegisterType::T7,
                RegisterType::T8,
                RegisterType::T9,
                RegisterType::At, // this one should go last.
            ],
        )?;
        for r in [
            RegisterType::Hi,
            RegisterType::Lo,
            RegisterType::V0,
            RegisterType::V1,
            RegisterType::T0,
            RegisterType::T1,
            RegisterType::T2,
            RegisterType::T3,
            RegisterType::T4,
            RegisterType::T5,
            RegisterType::T6,
            RegisterType::T7,
            RegisterType::T8,
            RegisterType::T9,
            RegisterType::At,
        ] {
            crate::dataflow::uninitialized(&self.seq, &r)?;
        }
        for r in [
            RegisterType::Hi,
            RegisterType::Lo,
            RegisterType::T0,
            RegisterType::T1,
            RegisterType::T2,
            RegisterType::T3,
            RegisterType::T4,
            RegisterType::T5,
            RegisterType::T6,
            RegisterType::T7,
            RegisterType::T8,
            RegisterType::T9,
            RegisterType::At,
        ] {
            crate::dataflow::dont_expect_write(&self.seq, &r)?;
        }
        for r in [
            RegisterType::S0,
            RegisterType::S1,
            RegisterType::S2,
            RegisterType::S3,
            RegisterType::S4,
            RegisterType::S5,
            RegisterType::S6,
            RegisterType::S7,
        ] {
            crate::dataflow::leave_alone(&self.seq, &r)?;
        }
        if let Some(arg) = self.args.get() {
            const ARGS: [RegisterType; 4] = [
                RegisterType::A0,
                RegisterType::A1,
                RegisterType::A2,
                RegisterType::A3,
            ];
            for a in &ARGS[..arg] {
                crate::dataflow::expect_read(&self.seq, a)?;
            }
            for a in &ARGS[arg..] {
                crate::dataflow::uninitialized(&self.seq, a)?;
            }
        }
        if let Some(val) = self.vals.get() {
            const VALS: [RegisterType; 2] = [RegisterType::V0, RegisterType::V1];
            for a in &VALS[..val] {
                crate::dataflow::expect_write(&self.seq, a)?;
            }
            for a in &VALS[val..] {
                crate::dataflow::dont_expect_write(&self.seq, a)?;
            }
        }
        crate::dataflow::expect_write(&self.seq, &RegisterType::V0)?;

        Ok(())
    }

    /// returns fixups ensuring compliance with the O32 calling convention.
    fn correctitudes(&self) -> crate::StaticAnalysis<Instruction> {
        use trapezoid_core::cpu::RegisterType;
        // make sure the sequence ends in a `jr $ra` instruction
        self.seq.check_last(Instruction::make_jr_ra)?;
        self.seq
            .check_all_but_last(Instruction::make_not_control_flow)?;
        for r in [
            RegisterType::K0,
            RegisterType::K1,
            RegisterType::Sp,
            RegisterType::Fp,
            RegisterType::Gp,
        ] {
            crate::dataflow::leave_alone(&self.seq, &r)?;
        }

        crate::dataflow::leave_alone_except_last(&self.seq, &RegisterType::Ra)?;

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

impl<Input: Parameters, Output: ReturnValue> std::fmt::Display for O32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.seq)
    }
}

impl<Input: Parameters, Output: ReturnValue> std::fmt::Debug for O32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.seq)
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Callable<Input, Output> for O32<Input, Output> {
    fn call(&self, parameters: Input) -> RunResult<Output> {
        let mut runner = O32Runner::new();
        parameters.put(&mut runner)?;
        self.args.set(Some(runner.arg as usize));
        runner.call_subroutine(&self.seq.to_bytes())?;
        let result = Output::get(&mut runner);
        self.vals.set(Some(runner.val as usize));
        result
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Traverse for O32<Input, Output> {
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

impl<Input: Parameters, Output: ReturnValue> Default for O32<Input, Output> {
    fn default() -> Self {
        Self {
            seq: crate::Sequence::<Instruction>::first(),
            _phantom_data: Default::default(),
            args: Default::default(),
            vals: Default::default(),
        }
    }
}

struct O32Runner {
    bus: crate::backends::mips::bus::Bus,
    cpu: trapezoid_core::cpu::Cpu,
    arg: u8,
    val: u8,
}

impl crate::test::TakeParameters for O32Runner {
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

impl crate::test::GetReturnValues for O32Runner {
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

impl O32Runner {
    pub fn new() -> Self {
        Self {
            cpu: trapezoid_core::cpu::Cpu::new(),
            bus: crate::backends::mips::bus::Bus::new(),
            arg: 0,
            val: 0,
        }
    }
    pub fn call_subroutine(&mut self, subroutine: &[u8]) -> RunResult<()> {
        self.bus.kseg1[0..subroutine.len()].copy_from_slice(subroutine);
        let end_pc = 0xBFC00000 + subroutine.len() as u32;
        for _ in 0..10000 {
            if self
                .cpu
                .registers()
                .read(trapezoid_core::cpu::RegisterType::Pc)
                == end_pc
            {
                return Ok(());
            }
            self.cpu.clock(&mut self.bus, 1);
        }
        Err(crate::RunError::RanAmok)
    }
    fn put_argument(&mut self, arg: u32) -> RunResult<()> {
        use trapezoid_core::cpu::RegisterType;
        match self.arg {
            0 => self.cpu.registers_mut().write(RegisterType::A0, arg),
            1 => self.cpu.registers_mut().write(RegisterType::A1, arg),
            2 => self.cpu.registers_mut().write(RegisterType::A2, arg),
            3 => self.cpu.registers_mut().write(RegisterType::A3, arg),
            _ => return Err(crate::RunError::TooManyArguments),
        }
        self.arg += 1;
        Ok(())
    }
    fn get_value(&mut self) -> RunResult<u32> {
        use trapezoid_core::cpu::RegisterType;
        let r = match self.val {
            0 => Ok(self.cpu.registers().read(RegisterType::V0)),
            1 => Ok(self.cpu.registers().read(RegisterType::V1)),
            _ => Err(crate::RunError::TooManyReturnValues),
        };
        self.val += 1;
        r
    }
}
