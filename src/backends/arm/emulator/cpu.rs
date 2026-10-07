use super::ExecutionError;
use super::Memory;
use super::RegisterFile;
use unarm::Ins;

/// Shamelessly stolen from armagnac
/// Calculate the sum of two 32-bit words and a carry bit, and calculate carry
/// and overflow status bits. Returns the tuple `(result, carry, overflow)`.
///
/// # Arguments
///
/// * `x` - First operand
/// * `y` - Second operand
/// * `carry_in` - Input carry bit
pub fn add_with_carry(x: u32, y: u32, carry_in: bool) -> (u32, bool, bool) {
    let carry_in = if carry_in { 1 } else { 0 };
    let unsigned_sum = x as u64 + y as u64 + carry_in as u64;
    let signed_sum = x as i32 as i64 + y as i32 as i64 + carry_in as i64;
    let result = x.wrapping_add(y).wrapping_add(carry_in as u32);
    let carry_out = result as u64 != unsigned_sum;
    let overflow = result as i32 as i64 != signed_sum;
    (result, carry_out, overflow)
}

pub struct Cpu {
    regs: RegisterFile,
    memory: Memory,
}

impl Cpu {
    pub fn single_step(&mut self) -> Result<(), ExecutionError> {
        let pc = self.regs.get(&unarm::Reg::Pc)?;
        if self.regs.is_thumb_mode() {
            todo!();
        } else {
            let instruction_word = self.memory.get(pc)?;
            let ins = unarm::parse_arm(instruction_word, pc, &Default::default());
            self.execute(&ins)
        }
    }

    pub fn cond(&self, cond: &unarm::Cond) -> bool {
        self.regs.cond(cond)
    }

    pub fn execute(&mut self, ins: &Ins) -> Result<(), ExecutionError> {
        if !self.cond(&ins.cond()) {
            return Ok(());
        }
        match ins {
            Ins::Add {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n + m;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Adc {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n + m + self.regs.c() as u32;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::And {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n & m;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Asr { .. } => todo!("ASR instruction apparently not present in ARMv4T"),
            Ins::B { cond, target } => {
                self.regs.set(&unarm::Reg::Pc, target.addr);
                Ok(())
            }
            Ins::Bic {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n & m;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Bkpt { imm } => Err(ExecutionError::Breakpoint(*imm as u8)),
            Ins::Bl { cond, target } => {
                self.regs
                    .set(&unarm::Reg::Lr, self.regs.get(&unarm::Reg::Pc)?);
                self.regs.set(&unarm::Reg::Pc, target.addr);
                // TODO: might need to switch to thumb mode or something
                Ok(())
            }
            Ins::Blx { cond, target } => {
                let target = match target {
                    unarm::BlxTarget::Direct(target) => target.addr,
                    unarm::BlxTarget::Indirect(reg) => self.regs.get(reg)?,
                };
                self.regs.set(&unarm::Reg::Pc, target);
                // TODO: might need to switch to thumb mode or something
                Ok(())
            }
            Ins::Bx { cond, rm } => {
                self.regs.set(&unarm::Reg::Pc, self.regs.get(rm)?);
                // TODO: might need to switch to thumb mode or something
                Ok(())
            }
            Ins::Bxj { .. } => unimplemented!("jazelle mode unimplemented (for now?)"),
            Ins::Cdp { .. } => unimplemented!("coprocessors unimplemented (for now?)"),
            Ins::Cdp2 { .. } => unimplemented!("coprocessors unimplemented (for now?)"),
            Ins::Clrex {} => todo!("what is Clrex"),
            Ins::Clz { cond, rd, rm } => {
                self.regs.set(rd, self.regs.get(rm)?.leading_zeros());
                Ok(())
            }
            Ins::Cmn { cond, rn, op2 } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let (result, carry, overflow) = add_with_carry(n, m, false);
                self.regs.set_z(result == 0);
                self.regs.set_c(carry);
                self.regs.set_v(overflow);
                Ok(())
            }
            Ins::Cmp { cond, rn, op2 } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let (result, carry, overflow) = add_with_carry(n, !m, false);
                self.regs.set_z(result == 0);
                self.regs.set_c(carry);
                self.regs.set_v(overflow);
                Ok(())
            }
            Ins::Cps { .. } => unimplemented!("what is cps"),
            Ins::Csdb { .. } => unimplemented!("what is csdb"),
            Ins::Dbg { .. } => unimplemented!("what is dbg"),
            Ins::Eor {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n ^ m;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Ldc { .. } => unimplemented!("coprocessors unimplemented (for now?)"),
            Ins::Ldc2 { .. } => unimplemented!("coprocessors unimplemented (for now?)"),
            Ins::Ldm {
                rn,
                mode,
                cond,
                writeback,
                regs,
                user_mode,
            } => {
                use unarm::LdmStmMode;
                let mut addr = self.regs.get(rn)?;
                for reg in regs.iter() {
                    let word = match mode {
                        LdmStmMode::Ia => {
                            // increment after
                            let word = self.memory.get(addr)?;
                            addr += 4;
                            word
                        }
                        LdmStmMode::Ib => {
                            // increment before
                            addr += 4;
                            self.memory.get(addr)?
                        }
                        LdmStmMode::Da => {
                            // decrement after
                            let word = self.memory.get(addr)?;
                            addr -= 4;
                            word
                        }
                        LdmStmMode::Db => {
                            // decrement before
                            addr -= 4;
                            self.memory.get(addr)?
                        }
                    };
                }
                if *writeback {
                    self.regs.set(rn, addr);
                }
                Ok(())
            }
            Ins::Ldr { cond, rd, addr } => {
                let addr = self.regs.get_addr_ldr_str(addr)?;
                self.regs.set(rd, addr);
                Ok(())
            }
        }
    }
}
