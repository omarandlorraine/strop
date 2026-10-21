use unarm::Op2;
use unarm::Reg;

/// Reason why it was impossible to set/
pub enum RegisterError {
    /// An error for if the bits in the CPSR don't encode a valid mode
    WrongModeBits,
    /// Trying to get a value but the register has never been written to
    UninitializedRegister,
}

#[derive(Default, Debug, Copy, Clone)]
struct Register {
    value: u32,
    initialized: bool,
}

impl Register {
    fn set(&mut self, val: u32) {
        self.value = val;
        self.initialized = true;
    }
    fn get(&self) -> Result<u32, RegisterError> {
        if self.initialized {
            Ok(self.value)
        } else {
            Err(RegisterError::UninitializedRegister)
        }
    }
}

#[derive(Default, Debug)]
struct Paged {
    usr: Register,
    svc: Register,
    abt: Register,
    und: Register,
    irq: Register,
    fiq: Register,
}

#[derive(Default, Debug)]
struct Fiqable {
    u: Register,
    f: Register,
}

impl Paged {
    pub fn set(&mut self, mode: Mode, val: u32) {
        match mode {
            Mode::User => self.usr.set(val),
            Mode::System => self.usr.set(val),
            Mode::Supervisor => self.svc.set(val),
            Mode::Fiq => self.fiq.set(val),
            Mode::Abort => self.abt.set(val),
            Mode::Irq => self.irq.set(val),
            Mode::Undefined => self.und.set(val),
        }
    }
    pub fn get(&self, mode: Mode) -> Register {
        match mode {
            Mode::User => self.usr,
            Mode::System => self.usr,
            Mode::Supervisor => self.svc,
            Mode::Fiq => self.fiq,
            Mode::Abort => self.abt,
            Mode::Irq => self.irq,
            Mode::Undefined => self.und,
        }
    }
}

impl Fiqable {
    pub fn get(&self, mode: Mode) -> Register {
        if mode == Mode::Fiq { self.f } else { self.u }
    }
    pub fn set(&mut self, mode: Mode, val: u32) {
        if mode == Mode::Fiq {
            self.f.set(val)
        } else {
            self.u.set(val)
        }
    }
}

#[derive(Debug)]
pub struct RegisterFile {
    r0: Register,
    r1: Register,
    r2: Register,
    r3: Register,
    r4: Register,
    r5: Register,
    r6: Register,
    r7: Register,
    r8: Fiqable,
    r9: Fiqable,
    r10: Fiqable,
    r11: Fiqable,
    r12: Fiqable,
    r13: Paged,
    r14: Paged,
    r15: Register,
    cpsr: u32,
    spsr: Paged,
}

impl RegisterFile {
    pub fn n(&self) -> bool {
        self.cpsr & 0x8000_0000 != 0
    }
    pub fn set_z(&mut self, v: bool) {
        if v {
            self.cpsr |= 0x4000_0000;
        } else {
            self.cpsr &= !0x4000_0000;
        }
    }
    pub fn z(&self) -> bool {
        self.cpsr & 0x4000_0000 != 0
    }
    pub fn c(&self) -> bool {
        self.cpsr & 0x2000_0000 != 0
    }
    pub fn set_c(&mut self, v: bool) {
        if v {
            self.cpsr |= 0x2000_0000;
        } else {
            self.cpsr &= !0x2000_0000;
        }
    }
    pub fn v(&self) -> bool {
        self.cpsr & 0x1000_0000 != 0
    }
    pub fn set_v(&mut self, v: bool) {
        if v {
            self.cpsr |= 0x1000_0000;
        } else {
            self.cpsr &= !0x1000_0000;
        }
    }

    pub fn cond(&self, cond: &unarm::Cond) -> bool {
        match cond {
            unarm::Cond::Al => true,
            unarm::Cond::Eq => self.z(),
            unarm::Cond::Ne => !self.z(),
            unarm::Cond::Mi => self.n(),
            unarm::Cond::Pl => !self.n(),
            unarm::Cond::Vs => self.v(),
            unarm::Cond::Vc => !self.v(),
            unarm::Cond::Hi => self.c() && !self.z(),
            unarm::Cond::Ls => !self.c() || self.z(),
            unarm::Cond::Ge => self.n() == self.v(),
            unarm::Cond::Lt => self.n() != self.v(),
            _ => todo!("unimplemented condition code {cond:?}"),
        }
    }
    pub fn is_thumb_mode(&self) -> bool {
        self.cpsr & 0x20 == 0x20
    }
    pub fn current_mode(&self) -> Result<Mode, RegisterError> {
        match self.cpsr & 0x1f {
            0x10 => Ok(Mode::User),
            0x11 => Ok(Mode::Fiq),
            0x12 => Ok(Mode::Irq),
            0x13 => Ok(Mode::Supervisor),
            0x17 => Ok(Mode::Abort),
            0x1b => Ok(Mode::Undefined),
            0x1f => Ok(Mode::System),
            _ => Err(RegisterError::WrongModeBits),
        }
    }

    pub fn shift_op(
        &self,
        imm: &u32,
        op: &unarm::ShiftOp,
        reg: &unarm::Reg,
    ) -> Result<u32, RegisterError> {
        use unarm::ShiftOp;
        let rm = self.get(reg)?;
        Ok(match op {
            ShiftOp::Lsl => rm << imm,
            ShiftOp::Lsr => rm >> imm,
            ShiftOp::Asr => ((rm as i32) << imm) as u32,
            ShiftOp::Ror => rm.rotate_right(*imm),
        })
    }

    pub fn get_offset(&mut self, o: &unarm::LdrStrOffset) -> Result<u32, RegisterError> {
        use unarm::LdrStrOffset;
        match o {
            LdrStrOffset::Imm(offset) => Ok(*offset as u32),
            LdrStrOffset::Reg {
                subtract,
                rm,
                shift_op,
                imm,
            } => {
                let offset = self.shift_op(imm, shift_op, rm)?;
                if *subtract {
                    Ok((0 - (offset as i32)) as u32)
                } else {
                    Ok(offset)
                }
            }
        }
    }

    pub fn get_misc_load_offset(
        &mut self,
        a: &unarm::MiscLoadOffset,
    ) -> Result<u32, RegisterError> {
        use unarm::MiscLoadOffset;
        match a {
            MiscLoadOffset::Imm(offs) => Ok(*offs as u32),
            MiscLoadOffset::Reg { subtract, rm } => {
                if *subtract {
                    Ok((0 - self.get(rm)? as i32) as u32)
                } else {
                    self.get(rm)
                }
            }
        }
    }

    pub fn get_addr_misc_load(&mut self, a: &unarm::AddrMiscLoad) -> Result<u32, RegisterError> {
        use unarm::AddrMiscLoad;
        match a {
            AddrMiscLoad::Pre {
                rn,
                offset,
                writeback,
            } => {
                let addr = self.get(rn)? + self.get_misc_load_offset(offset)?;
                if *writeback {
                    self.set(rn, addr);
                }
                Ok(addr)
            }
            AddrMiscLoad::Post { rn, offset } => {
                let addr = self.get(rn)? + self.get_misc_load_offset(offset)?;
                self.set(rn, addr);
                Ok(addr)
            }
        }
    }

    pub fn get_addr_ldr_str_post(
        &mut self,
        a: &unarm::AddrLdrStrPost,
    ) -> Result<u32, RegisterError> {
        let unarm::AddrLdrStrPost { rn, offset } = a;
        Ok(self.get(rn)?.wrapping_add(self.get_offset(offset)?))
    }

    pub fn get_addr_ldr_str(&mut self, a: &unarm::AddrLdrStr) -> Result<u32, RegisterError> {
        use unarm::AddrLdrStr;
        use unarm::LdrStrOffset;
        match a {
            AddrLdrStr::Post(post) => self.get_addr_ldr_str_post(post),
            AddrLdrStr::Pre {
                rn,
                offset,
                writeback,
            } => {
                let address = self.get(rn)?.wrapping_add(self.get_offset(offset)? as u32);
                if *writeback {
                    self.set(rn, address);
                }
                Ok(address)
            }
        }
    }

    pub fn op2shift(&self, o: &unarm::Op2Shift) -> Result<u32, RegisterError> {
        use unarm::Op2Imm;
        use unarm::Op2Shift;
        use unarm::ShiftImm;
        use unarm::ShiftOp;
        use unarm::ShiftReg;
        match o {
            Op2Shift::Imm(imm) => Ok(*imm),
            Op2Shift::Reg(r) => self.get(r),
        }
    }

    pub fn op2(&self, o: &Op2) -> Result<u32, RegisterError> {
        use unarm::Op2Imm;
        use unarm::ShiftImm;
        use unarm::ShiftOp;
        use unarm::ShiftReg;
        match o {
            Op2::Imm(Op2Imm { imm, rotate_imm: _ }) => Ok(*imm),
            Op2::ShiftImm(ShiftImm { rm, imm, shift_op }) => self.shift_op(imm, shift_op, rm),
            Op2::ShiftReg(ShiftReg { rm, rs, shift_op }) => {
                self.shift_op(&self.get(rs)?, shift_op, rm)
            }
        }
    }

    pub fn get(&self, reg: &Reg) -> Result<u32, RegisterError> {
        match reg {
            Reg::R0 => self.r0.get(),
            Reg::R1 => self.r1.get(),
            Reg::R2 => self.r2.get(),
            Reg::R3 => self.r3.get(),
            Reg::R4 => self.r4.get(),
            Reg::R5 => self.r5.get(),
            Reg::R6 => self.r6.get(),
            Reg::R7 => self.r7.get(),
            Reg::R8 => self.r8.get(self.current_mode()?).get(),
            Reg::R9 => self.r9.get(self.current_mode()?).get(),
            Reg::R10 => self.r10.get(self.current_mode()?).get(),
            Reg::R11 => self.r11.get(self.current_mode()?).get(),
            Reg::R12 => self.r12.get(self.current_mode()?).get(),
            Reg::Sp => self.r13.get(self.current_mode()?).get(),
            Reg::Lr => self.r14.get(self.current_mode()?).get(),
            Reg::Pc => self.r15.get(),
        }
    }

    pub fn getlong(&mut self, reg_hi: &Reg, reg_lo: &Reg) -> Result<i64, RegisterError> {
        let a_lo = self.get(reg_lo)? as u64;
        let a_hi = self.get(reg_hi)? as u64;
        Ok((a_hi << 32 | a_lo) as i64)
    }

    pub fn setlong(&mut self, reg_hi: &Reg, reg_lo: &Reg, val: i64) -> Result<(), RegisterError> {
        let val_hi = (val >> 32) as u32;
        let val_lo = (val & 0x0000_0000_ffff_ffff) as u32;
        self.set(reg_lo, val_lo)?;
        self.set(reg_hi, val_hi)
    }

    pub fn set(&mut self, reg: &Reg, val: u32) -> Result<(), RegisterError> {
        match reg {
            Reg::R0 => self.r0.set(val),
            Reg::R1 => self.r1.set(val),
            Reg::R2 => self.r2.set(val),
            Reg::R3 => self.r3.set(val),
            Reg::R4 => self.r4.set(val),
            Reg::R5 => self.r5.set(val),
            Reg::R6 => self.r6.set(val),
            Reg::R7 => self.r7.set(val),
            Reg::R8 => self.r8.set(self.current_mode()?, val),
            Reg::R9 => self.r9.set(self.current_mode()?, val),
            Reg::R10 => self.r10.set(self.current_mode()?, val),
            Reg::R11 => self.r11.set(self.current_mode()?, val),
            Reg::R12 => self.r12.set(self.current_mode()?, val),
            Reg::Sp => self.r13.set(self.current_mode()?, val),
            Reg::Lr => self.r14.set(self.current_mode()?, val),
            Reg::Pc => self.r15.set(val),
        };
        Ok(())
    }
}

/// An ARMv4T processor mode. This was shamelessly stolen from daniel5151/armv4t_emu
/// and I'm assuming it's the same for arm5 and 6
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
    User,
    Fiq,
    Irq,
    Supervisor,
    Abort,
    Undefined,
    System,
}

impl Mode {
    #[inline]
    pub(crate) fn from_bits(mode: u8) -> Option<Self> {
        use self::Mode::*;
        Some(match mode & 0x1f {
            0x10 => User,
            0x11 => Fiq,
            0x12 => Irq,
            0x13 => Supervisor,
            0x17 => Abort,
            0x1b => Undefined,
            0x1f => System,
            _ => return None,
        })
    }

    #[inline]
    pub(crate) fn bits(self) -> u8 {
        use self::Mode::*;
        match self {
            User => 0x10,
            Fiq => 0x11,
            Irq => 0x12,
            Supervisor => 0x13,
            Abort => 0x17,
            Undefined => 0x1b,
            System => 0x1f,
        }
    }

    pub(crate) fn is_fiq(&self) -> bool {
        matches!(self, Self::Fiq)
    }
}
