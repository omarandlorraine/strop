//! Module implementing O32, a Callable and Searchable complying with MIPS's O32 calling
//! convention.
//!
//! This is good for searching for pure leaf O32 functions that take at least one argument and
//! return at least one thing.

use crate::RunResult;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::backends::mips::Instruction;
use crate::test::{Parameters, ReturnValue};
use trapezoid_core::cpu::RegisterType;

fn allocate_temporary_registers(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    crate::dataflow::allocate_registers(
        seq,
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
    )
}

fn leave_callee_saved_registers_alone(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
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
        crate::dataflow::leave_alone(seq, &r)?;
    }
    Ok(())
}

fn expect_one_parameter(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    crate::dataflow::expect_read(seq, &RegisterType::A0)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A1)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A2)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A3)
}

fn expect_two_parameters(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    crate::dataflow::expect_read(seq, &RegisterType::A0)?;
    crate::dataflow::expect_read(seq, &RegisterType::A1)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A2)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A3)
}

fn expect_three_parameters(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    crate::dataflow::expect_read(seq, &RegisterType::A0)?;
    crate::dataflow::expect_read(seq, &RegisterType::A1)?;
    crate::dataflow::expect_read(seq, &RegisterType::A2)?;
    crate::dataflow::uninitialized(seq, &RegisterType::A3)
}

fn expect_four_parameters(seq: &Sequence<Instruction>) -> StaticAnalysis<Instruction> {
    crate::dataflow::expect_read(seq, &RegisterType::A0)?;
    crate::dataflow::expect_read(seq, &RegisterType::A1)?;
    crate::dataflow::expect_read(seq, &RegisterType::A2)?;
    crate::dataflow::expect_read(seq, &RegisterType::A3)
}

/// Searches for functions complying to the O32 calling convention
#[derive(Clone)]
pub struct O32<Input: Parameters, Output: ReturnValue> {
    subroutine: crate::search::platform_specific::PlatformSpecificSequence<Instruction>,
    vals: std::cell::Cell<Option<usize>>,
    _phantom_data: std::marker::PhantomData<(Input, Output)>,
}

impl<Input: Parameters, Output: ReturnValue> O32<Input, Output> {
    /*
            self.seq
                .check_all(Instruction::make_not_redundantly_encoded)?;

    */

    /*
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
    */

    /*
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
    */

    /*
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

    */

    /*
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

    self.make_correct();
    while let Err(fixup) = self.reduce_search_space() {
        self.seq.apply(&fixup);
        self.make_correct();
    }
    */
}

impl<Input: Parameters, Output: ReturnValue> std::fmt::Display for O32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.subroutine)
    }
}

impl<Input: Parameters, Output: ReturnValue> std::fmt::Debug for O32<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.subroutine)
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Callable<Input, Output> for O32<Input, Output> {
    fn call(&self, parameters: Input) -> RunResult<Output> {
        let mut runner = parameters.into_mips_o32_runner()?;
        runner.call_subroutine(&self.subroutine.to_bytes())?;
        let result = Output::get(&mut runner);
        self.vals.set(Some(runner.val as usize));
        result
    }
}

impl<Input: Parameters, Output: ReturnValue> crate::Traverse for O32<Input, Output> {
    fn increment(&mut self) {
        self.subroutine.increment();
    }
    fn mutate(&mut self) {
        self.subroutine.mutate();
    }
    fn from_bytes(_bytes: &[u8]) -> Option<Self> {
        todo!()
    }
}

impl<Input: Parameters, Output: ReturnValue> O32<Input, Output> {
    /// Instantiates the searcher, if possible.
    pub fn new() -> crate::RunResult<Self> {
        let subroutine = crate::backends::mips::subroutine()
            .extend(Input::default().into_mips_o32_runner()?.args)
            .add(allocate_temporary_registers)
            .add(leave_callee_saved_registers_alone)
            .to_owned();
        Ok(Self {
            subroutine,
            _phantom_data: Default::default(),
            vals: Default::default(),
        })
    }
}

/// Runs a MIPS subroutine; also conveniences O32 argument passing
pub struct Runner {
    bus: crate::backends::mips::bus::Bus,
    cpu: trapezoid_core::cpu::Cpu,
    /// Static analysis passes correcting dataflow into the function
    pub args: &'static [fn(&Sequence<Instruction>) -> StaticAnalysis<Instruction>],
    val: u8,
}

impl std::fmt::Debug for Runner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner").finish()
    }
}

impl crate::test::GetReturnValues for Runner {
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

impl Runner {
    /// Instantiates a Runner with one argument in place
    pub fn new1(a0: u32) -> Self {
        let mut cpu = trapezoid_core::cpu::Cpu::new();
        cpu.registers_mut().write(RegisterType::A0, a0);
        Self {
            cpu,
            bus: crate::backends::mips::bus::Bus::new(),
            args: &[expect_one_parameter],
            val: 0,
        }
    }
    /// Instantiates a Runner with two arguments in place
    pub fn new2(a0: u32, a1: u32) -> Self {
        let mut cpu = trapezoid_core::cpu::Cpu::new();
        cpu.registers_mut().write(RegisterType::A0, a0);
        cpu.registers_mut().write(RegisterType::A1, a1);
        Self {
            cpu,
            bus: crate::backends::mips::bus::Bus::new(),
            args: &[expect_two_parameters],
            val: 0,
        }
    }
    /// Instantiates a Runner with three arguments in place
    pub fn new3(a0: u32, a1: u32, a2: u32) -> Self {
        let mut cpu = trapezoid_core::cpu::Cpu::new();
        cpu.registers_mut().write(RegisterType::A0, a0);
        cpu.registers_mut().write(RegisterType::A1, a1);
        cpu.registers_mut().write(RegisterType::A2, a2);
        Self {
            cpu,
            bus: crate::backends::mips::bus::Bus::new(),
            args: &[expect_three_parameters],
            val: 0,
        }
    }
    /// Instantiates a Runner with four arguments in place
    pub fn new4(a0: u32, a1: u32, a2: u32, a3: u32) -> Self {
        let mut cpu = trapezoid_core::cpu::Cpu::new();
        cpu.registers_mut().write(RegisterType::A0, a0);
        cpu.registers_mut().write(RegisterType::A1, a1);
        cpu.registers_mut().write(RegisterType::A2, a2);
        cpu.registers_mut().write(RegisterType::A3, a3);
        Self {
            cpu,
            bus: crate::backends::mips::bus::Bus::new(),
            args: &[expect_four_parameters],
            val: 0,
        }
    }
    /// writes a subroutine to the beginning of kseg1, and then calls it
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
