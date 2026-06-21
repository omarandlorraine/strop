use super::Variant;
use super::thumb::ThumbInstruction;
use crate::RunResult;
use crate::test::Parameters;
use crate::test::ReturnValue;

/// Searches for functions complying with the AAPCS32 calling convention.
pub struct Aapcs32<V: Variant, Input: Parameters, Output: ReturnValue> {
    seq: crate::Sequence<ThumbInstruction<V>>,
    _phantom: (
        std::marker::PhantomData<Input>,
        std::marker::PhantomData<Output>,
    ),
    args: std::cell::Cell<Option<usize>>,
    vals: std::cell::Cell<Option<usize>>,
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> std::fmt::Display
    for Aapcs32<V, Input, Output>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.seq)
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> std::fmt::Debug
    for Aapcs32<V, Input, Output>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.seq)
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> Aapcs32<V, Input, Output> {
    fn correctitudes(&self) -> crate::StaticAnalysis<ThumbInstruction<V>> {
        self.seq.check_last(ThumbInstruction::make_bx_lr)?;
        Ok(())
    }
    fn make_correct(&mut self) {
        while let Err(fixup) = self.correctitudes() {
            self.seq.apply(&fixup);
        }
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> Default for Aapcs32<V, Input, Output> {
    fn default() -> Self {
        Self {
            seq: crate::Sequence::first(),
            _phantom: Default::default(),
            args: Default::default(),
            vals: Default::default(),
        }
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> crate::Traverse
    for Aapcs32<V, Input, Output>
{
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
            seq: crate::Sequence::from_bytes(bytes)?,
            ..Default::default()
        })
    }
}

impl<V: Variant, Input: Parameters, Output: ReturnValue> crate::Callable<Input, Output>
    for Aapcs32<V, Input, Output>
{
    fn call(&self, parameters: Input) -> RunResult<Output> {
        let mut runner = Aapcs32Runner::<V>::new();
        parameters.put(&mut runner)?;
        self.args.set(Some(runner.arg));
        runner.call_subroutine(&self.seq)?;
        let result = Output::get(&mut runner);
        self.vals.set(Some(runner.val));
        result
    }
}

/// A basic emulator
pub struct Aapcs32Runner<V: Variant> {
    /// The emulated system's CPU
    pub proc: armagnac::core::Processor,

    pub arg: usize,
    pub val: usize,

    _phantom: std::marker::PhantomData<V>,
}

impl<V: Variant> crate::test::TakeParameters for Aapcs32Runner<V> {
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

impl From<armagnac::core::MapConflict> for crate::RunError {
    fn from(_mc: armagnac::core::MapConflict) -> Self {
        todo!()
    }
}

impl From<armagnac::core::RunError> for crate::RunError {
    fn from(re: armagnac::core::RunError) -> Self {
        dbg!(re);
        crate::RunError::RanAmok
    }
}

impl<V: Variant> Aapcs32Runner<V> {
    pub fn new() -> Self {
        Self {
            proc: V::proc(),
            arg: 0,
            val: 0,
            _phantom: Default::default(),
        }
    }
    pub fn call_subroutine(
        &mut self,
        subroutine: &crate::Sequence<ThumbInstruction<V>>,
    ) -> RunResult<()> {
        use crate::RunError;
        use armagnac::core::Emulator;

        const START_OF_SUBROUTINE: u32 = 0x2000;

        self.proc.map(START_OF_SUBROUTINE, &subroutine.to_bytes())?;
        self.proc.set_pc(START_OF_SUBROUTINE);

        const RETURN_ADDRESS: u32 = 0x5678;
        const BOTTOM_OF_STACK: u32 = 0x0100;

        self.proc.set_sp(BOTTOM_OF_STACK);
        self.proc.set_lr(RETURN_ADDRESS);

        let end_of_subroutine = START_OF_SUBROUTINE + subroutine.to_bytes().len() as u32;

        for _ in 0..10000 {
            let pc = self.proc.registers.pc;
            let sp = self.proc.registers.sp();

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
            self.proc.run(armagnac::core::RunOptions::new().gas(1))?;
        }

        Err(crate::RunError::RanAmok)
    }
    fn put_argument(&mut self, arg: u32) -> RunResult<()> {
        match self.arg {
            0 => self.proc.registers.r0 = arg,
            1 => self.proc.registers.r1 = arg,
            2 => self.proc.registers.r2 = arg,
            3 => self.proc.registers.r3 = arg,
            _ => return Err(crate::RunError::TooManyArguments),
        }
        self.arg += 1;
        Ok(())
    }
    fn get_value(&mut self) -> RunResult<u32> {
        let r = match self.val {
            0 => Ok(self.proc.registers.r0),
            1 => Ok(self.proc.registers.r1),
            _ => Err(crate::RunError::TooManyReturnValues),
        };
        self.val += 1;
        r
    }
}

impl<V: Variant> crate::test::GetReturnValues for Aapcs32Runner<V> {
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
