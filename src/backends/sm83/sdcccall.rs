use crate::RunResult;
use crate::StaticAnalysis;
use crate::backends::sm83;
use crate::backends::x80::EmuInterface;
use crate::backends::x80::data::Datum;
use sm83::Emulator;
use sm83::Instruction;

#[derive(Debug, Default)]
pub struct SdccRunner {
    emu: Emulator,
    vals: Vec<Datum>,
}

impl SdccRunner {
    fn already_got(&self) -> RunResult<()> {
        if self.vals.is_empty() {
            Ok(())
        } else {
            Err(crate::RunError::TooManyReturnValues)
        }
    }
}

impl crate::test::GetReturnValues for SdccRunner {
    fn get_bool(&mut self) -> RunResult<bool> {
        Err(crate::RunError::UnsupportedType)
    }
    fn get_i8(&mut self) -> RunResult<i8> {
        Ok(self.get_u8()? as i8)
    }
    fn get_u8(&mut self) -> RunResult<u8> {
        self.already_got()?;
        self.vals.push(Datum::A);
        Ok(self.emu.get_a())
    }
    fn get_i16(&mut self) -> RunResult<i16> {
        Ok(self.get_u16()? as i16)
    }
    fn get_u16(&mut self) -> RunResult<u16> {
        self.already_got()?;
        self.vals.push(Datum::B);
        self.vals.push(Datum::C);
        Ok(self.emu.get_bc())
    }
    fn get_i32(&mut self) -> RunResult<i32> {
        Ok(self.get_u32()? as i32)
    }
    fn get_u32(&mut self) -> RunResult<u32> {
        self.already_got()?;
        self.vals.push(Datum::B);
        self.vals.push(Datum::C);
        self.vals.push(Datum::D);
        self.vals.push(Datum::E);
        Ok(u32::from_be_bytes([
            self.emu.get_d(),
            self.emu.get_e(),
            self.emu.get_b(),
            self.emu.get_c(),
        ]))
    }
    fn get_f32(&mut self) -> RunResult<f32> {
        Err(crate::RunError::UnsupportedType)
    }
}

impl crate::test::TakeParameters for SdccRunner {
    fn put_bool(&mut self, _v: bool) -> RunResult<()> {
        Err(crate::RunError::UnsupportedType)
    }
    fn put_i8(&mut self, v: i8) -> RunResult<()> {
        self.put_u8(v as u8)
    }
    fn put_u8(&mut self, v: u8) -> RunResult<()> {
        if !self.emu.reg_init.a.is_initialized() {
            self.emu.set_a(v);
            Ok(())
        } else if !self.emu.reg_init.e.is_initialized() {
            self.emu.set_e(v);
            Ok(())
        } else {
            // TODO: it doesn't fit in CPU registers, put it on the stack!
            Err(crate::RunError::TooManyArguments)
        }
    }
    fn put_i16(&mut self, v: i16) -> RunResult<()> {
        self.put_u16(v as u16)
    }
    fn put_u16(&mut self, v: u16) -> RunResult<()> {
        if !self.emu.reg_init.b.is_initialized() {
            self.emu.set_bc(v);
            Ok(())
        } else if !self.emu.reg_init.d.is_initialized() {
            self.emu.set_de(v);
            Ok(())
        } else {
            // TODO: it doesn't fit in CPU registers, put it on the stack!
            Err(crate::RunError::TooManyArguments)
        }
    }
    fn put_i32(&mut self, v: i32) -> RunResult<()> {
        self.put_u32(v as u32)
    }
    fn put_u32(&mut self, v: u32) -> RunResult<()> {
        if !self.emu.reg_init.d.is_initialized() {
            let v = v.to_be_bytes();
            self.emu.set_d(v[0]);
            self.emu.set_e(v[1]);
            self.emu.set_b(v[2]);
            self.emu.set_c(v[3]);
            Ok(())
        } else {
            // TODO: it doesn't fit in CPU registers, put it on the stack!
            Err(crate::RunError::TooManyArguments)
        }
    }
    fn put_f32(&mut self, _v: f32) -> RunResult<()> {
        Err(crate::RunError::UnsupportedType)
    }
}

/// A type representing a subroutine mimicking the calling convention used by modern-day SDCC.
/// SDCC's internal documentation calls this `__sdcccall(1)`.
#[derive(Default)]
pub struct SdccCall1 {
    seq: crate::Sequence<Instruction>,
}

impl std::fmt::Display for SdccCall1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.seq)
    }
}

impl std::fmt::Debug for SdccCall1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.seq)
    }
}

impl SdccCall1 {
    fn analyze(&self) -> StaticAnalysis<Instruction> {
        self.seq.check_last(Instruction::make_return)?;
        Ok(())
    }
    fn make_correct(&mut self) {
        while let Err(fixup) = self.analyze() {
            self.seq.apply(&fixup);
        }
    }
}

impl crate::Traverse for SdccCall1 {
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
        })
    }
}

impl<Params: crate::test::Parameters, RetVal: crate::test::ReturnValue>
    crate::Callable<Params, RetVal> for SdccCall1
{
    fn call(&self, input: Params) -> crate::RunResult<RetVal> {
        let mut emu = SdccRunner::default();
        input.put(&mut emu)?;
        emu.emu.call(self.seq.to_bytes())?;
        let r = RetVal::get(&mut emu)?;
        Ok(r)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn cfg() {
        use crate::Traverse;
        use crate::backends::sm83::SdccCall1;
        let mut c = SdccCall1::default();

        for _ in 0..5 {
            println!("{c}");
            c.increment();
        }
    }
}
