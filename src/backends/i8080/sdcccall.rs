use crate::RunResult;
use crate::backends::i8080;
use crate::backends::x80::EmuInterface;
use crate::backends::x80::data::Datum;
use i8080::Emulator;
use i8080::Instruction;

#[derive(Debug, Default)]
pub struct SdccRunner {
    emu: Emulator,
    params: Vec<Datum>,
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

impl crate::backends::x80::sdcccall1::SdccCall1PushPop for SdccRunner {
    fn input_registers(&self) -> Vec<Datum> {
        self.params.clone()
    }
    fn output_registers(&self) -> Vec<Datum> {
        self.vals.clone()
    }
}

impl EmuInterface for SdccRunner {
    fn set_a(&mut self, val: u8) {
        self.params.push(Datum::A);
        self.emu.set_a(val);
    }
    fn set_b(&mut self, val: u8) {
        self.params.push(Datum::B);
        self.emu.set_b(val);
    }
    fn set_c(&mut self, val: u8) {
        self.params.push(Datum::C);
        self.emu.set_c(val);
    }
    fn set_d(&mut self, val: u8) {
        self.params.push(Datum::D);
        self.emu.set_d(val);
    }
    fn set_e(&mut self, val: u8) {
        self.params.push(Datum::E);
        self.emu.set_e(val);
    }
    fn set_h(&mut self, val: u8) {
        self.params.push(Datum::H);
        self.emu.set_h(val);
    }
    fn set_l(&mut self, val: u8) {
        self.params.push(Datum::L);
        self.emu.set_l(val);
    }
    fn get_a(&self) -> u8 {
        self.emu.get_a()
    }
    fn get_b(&self) -> u8 {
        self.emu.get_b()
    }
    fn get_c(&self) -> u8 {
        self.emu.get_c()
    }
    fn get_d(&self) -> u8 {
        self.emu.get_d()
    }
    fn get_e(&self) -> u8 {
        self.emu.get_e()
    }
    fn get_hl(&self) -> u16 {
        self.emu.get_hl()
    }
    fn get_pc(&self) -> u16 {
        self.emu.get_pc()
    }
    fn get_sp(&self) -> u16 {
        self.emu.get_sp()
    }
    fn push(&mut self, val: u16) {
        self.emu.push(val);
    }
    fn pop(&mut self) -> u16 {
        self.emu.pop()
    }
    fn call(&mut self, seq: Vec<u8>) -> crate::RunResult<()> {
        self.emu.call(seq)
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
        Ok(self.get_a())
    }
    fn get_i16(&mut self) -> RunResult<i16> {
        Ok(self.get_u16()? as i16)
    }
    fn get_u16(&mut self) -> RunResult<u16> {
        self.already_got()?;
        self.vals.push(Datum::D);
        self.vals.push(Datum::E);
        Ok(u16::from_le_bytes([self.get_d(), self.get_e()]))
    }
    fn get_i32(&mut self) -> RunResult<i32> {
        Ok(self.get_u32()? as i32)
    }
    fn get_u32(&mut self) -> RunResult<u32> {
        self.already_got()?;
        self.vals.push(Datum::H);
        self.vals.push(Datum::L);
        self.vals.push(Datum::D);
        self.vals.push(Datum::E);
        Ok(u32::from_be_bytes([
            self.get_h(),
            self.get_l(),
            self.get_d(),
            self.get_e(),
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
        if !self.params.contains(&Datum::A) {
            self.set_a(v);
            Ok(())
        } else if !self.params.contains(&Datum::L) {
            self.set_l(v);
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
        if !self.params.contains(&Datum::A) && !self.params.contains(&Datum::H) {
            self.set_hl(v);
            Ok(())
        } else if !self.params.contains(&Datum::D) {
            self.set_de(v);
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
        if self.params.is_empty() {
            let v = v.to_be_bytes();
            self.set_h(v[0]);
            self.set_l(v[1]);
            self.set_d(v[2]);
            self.set_e(v[3]);
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

impl crate::backends::x80::SdccCallable for Instruction {
    type Runner = SdccRunner;
}

#[cfg(test)]
mod test {
    #[test]
    fn cfg() {
        use crate::Traverse;
        use crate::backends::i8080::Instruction;
        use crate::backends::x80::SdccCall1;
        let mut c = SdccCall1::<Instruction>::default();

        for _ in 0..5 {
            println!("{c}");
            c.increment();
        }
    }
}
