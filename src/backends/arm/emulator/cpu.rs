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
            let instruction_word = self.memory.get32(pc)?;
            let ins = unarm::parse_arm(instruction_word, pc, &Default::default());
            self.execute(&ins)
        }
    }

    pub fn cond(&self, cond: &unarm::Cond) -> bool {
        self.regs.cond(cond)
    }

    pub fn execute(&mut self, ins: &Ins) -> Result<(), ExecutionError> {
        fn to_halfwords(w: u32) -> (i16, i16) {
            ((w & 0x0000_ffff) as i16, (w >> 16) as i16)
        }
        fn from_halfwords(left: i16, right: i16) -> u32 {
            (right as u32 | (left as u32) << 16)
        }
        fn to_bytes(w: u32) -> (i8, i8, i8, i8) {
            (
                (w & 0x0000_00ff) as i8,
                (w >> 8) as i8,
                (w >> 16) as i8,
                (w >> 24) as i8,
            )
        }
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
                // TODO: what is user_mode?
                use unarm::LdmStmMode;
                let mut addr = self.regs.get(rn)?;
                for reg in regs.iter() {
                    let word = match mode {
                        LdmStmMode::Ia => {
                            // increment after
                            let word = self.memory.get32(addr)?;
                            addr += 4;
                            word
                        }
                        LdmStmMode::Ib => {
                            // increment before
                            addr += 4;
                            self.memory.get32(addr)?
                        }
                        LdmStmMode::Da => {
                            // decrement after
                            let word = self.memory.get32(addr)?;
                            addr -= 4;
                            word
                        }
                        LdmStmMode::Db => {
                            // decrement before
                            addr -= 4;
                            self.memory.get32(addr)?
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
                self.regs.set(rd, self.memory.get32(addr)?);
                Ok(())
            }
            Ins::Ldrb { cond, rd, addr } => {
                let addr = self.regs.get_addr_ldr_str(addr)?;
                self.regs.set(rd, self.memory.get8(addr)? as u32);
                Ok(())
            }
            Ins::Ldrbt { cond, rd, addr } => {
                // TODO: what is the difference between this and ldrb?
                let addr = self.regs.get_addr_ldr_str_post(addr)?;
                self.regs.set(rd, self.memory.get8(addr)? as u32);
                Ok(())
            }
            Ins::Ldrd {
                cond,
                rd,
                rd2,
                addr,
            } => {
                let addr = self.regs.get_addr_misc_load(addr)?;
                self.regs.set(rd, self.memory.get32(addr)?);
                self.regs.set(rd2, self.memory.get32(addr + 4)?);
                Ok(())
            }
            // these appear to be synchronization primitives
            Ins::Ldrex { .. } | Ins::Ldrexb { .. } | Ins::Ldrexd { .. } | Ins::Ldrexh { .. } => {
                todo!()
            }
            Ins::Ldrh { cond, rd, addr } => {
                let addr = self.regs.get_addr_misc_load(addr)?;
                self.regs.set(rd, self.memory.get16(addr)? as u32);
                Ok(())
            }
            Ins::Ldrsb { cond, rd, addr } => {
                let addr = self.regs.get_addr_misc_load(addr)?;
                self.regs
                    .set(rd, self.memory.get8(addr)? as i8 as i32 as u32);
                Ok(())
            }
            Ins::Ldrsh { cond, rd, addr } => {
                let addr = self.regs.get_addr_misc_load(addr)?;
                self.regs
                    .set(rd, self.memory.get16(addr)? as i16 as i32 as u32);
                Ok(())
            }
            Ins::Ldrt { cond, rd, addr } => {
                // TODO: what is the difference between this and ldr?
                let addr = self.regs.get_addr_ldr_str_post(addr)?;
                self.regs.set(rd, self.memory.get32(addr)?);
                Ok(())
            }
            Ins::Lsl {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                self.regs.set(
                    rd,
                    self.regs
                        .shift_op(&self.regs.op2shift(op2)?, &unarm::ShiftOp::Lsl, rd)?,
                );
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Lsr {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                self.regs.set(
                    rd,
                    self.regs
                        .shift_op(&self.regs.op2shift(op2)?, &unarm::ShiftOp::Lsr, rd)?,
                );
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Mcr { .. } | Ins::Mcrr { .. } | Ins::Mcrr2 { .. } | Ins::Mcr2 { .. } => {
                unimplemented!()
            }
            Ins::Mla {
                s,
                cond,
                rd,
                rn,
                rm,
                ra,
            } => {
                self.regs.set(
                    rd,
                    self.regs
                        .get(rn)?
                        .wrapping_mul(self.regs.get(rm)?)
                        .wrapping_add(self.regs.get(ra)?),
                );
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Mov {
                s,
                thumb,
                cond,
                rd,
                op2,
            } => {
                self.regs.set(rd, self.regs.op2(op2)?);
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Mrc { .. } | Ins::Mrrc { .. } | Ins::Mrrc2 { .. } | Ins::Mrc2 { .. } => {
                unimplemented!()
            }
            Ins::Mrs { .. } | Ins::Msr { .. } => unimplemented!(),
            Ins::Mul {
                s,
                thumb,
                cond,
                rd,
                rn,
                rm,
            } => {
                self.regs
                    .set(rd, self.regs.get(rn)?.wrapping_mul(self.regs.get(rm)?));
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Mvn {
                s,
                thumb,
                cond,
                rd,
                op2,
            } => {
                self.regs.set(rd, !self.regs.op2(op2)?);
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Neg { rd, rm } => {
                // TODO: what's the difference between this and mvn?
                self.regs.set(rd, !self.regs.get(rm)?);
                Ok(())
            }
            Ins::Nop { cond } => Ok(()),
            Ins::Orr {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n | m;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Pkhbt {
                cond,
                rd,
                rn,
                rm,
                shift_op,
                shift,
            } => {
                let shifted_rm = self.regs.shift_op(shift, shift_op, rm)?;
                self.regs.set(
                    rd,
                    (self.regs.get(rn)? & 0x0000_ffff) | (shifted_rm & 0xffff_0000),
                );
                Ok(())
            }
            Ins::Pkhtb {
                cond,
                rd,
                rn,
                rm,
                shift_op,
                shift,
            } => {
                let shifted_rm = self.regs.shift_op(shift, shift_op, rm)?;
                self.regs.set(
                    rd,
                    (self.regs.get(rn)? & 0xffff_0000) | (shifted_rm & 0x0000_ffff),
                );
                Ok(())
            }
            Ins::Pld { .. } => {
                // a preload instructions.
                // don't think there's much need to emulate this, so I'm just doing NOP here.
                Ok(())
            }
            Ins::Pop { cond, regs } => {
                // TODO: what is user_mode?
                self.execute(&Ins::Ldm {
                    cond: *cond,
                    user_mode: true,
                    writeback: true,
                    rn: unarm::Reg::Sp,
                    mode: unarm::LdmStmMode::Ia,
                    regs: *regs,
                })
            }
            Ins::Push { cond, regs } => {
                // TODO: what is user_mode?
                self.execute(&Ins::Stm {
                    cond: *cond,
                    user_mode: true,
                    writeback: true,
                    rn: unarm::Reg::Sp,
                    mode: unarm::LdmStmMode::Ia,
                    regs: *regs,
                })
            }
            Ins::Qadd { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                self.regs
                    .set(rd, self.regs.get(rn)?.saturating_add(self.regs.get(rm)?));
                Ok(())
            }
            Ins::Qadd16 { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.0.saturating_add(rn.0), rm.1.saturating_add(rn.1)),
                );
                Ok(())
            }
            Ins::Qadd8 { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                let rm = to_bytes(self.regs.get(rm)?);
                let rn = to_bytes(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    (rm.0.saturating_add(rn.0) as u32
                        | (rn.1.saturating_add(rn.1) as u32) << 8
                        | (rn.2.saturating_add(rn.2) as u32) << 16
                        | (rn.3.saturating_add(rn.3) as u32) << 24),
                );
                Ok(())
            }
            Ins::Qasx { cond, rd, rn, rm } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.1.saturating_add(rn.0), rm.0.saturating_sub(rn.1)),
                );
                Ok(())
            }
            Ins::Qdadd { cond, rd, rm, rn } => {
                // The Q flag is not handled corerctly (or at all) here
                let doubled = self.regs.get(rd)?.saturating_mul(2);
                let added = self.regs.get(rm)?.saturating_add(doubled);
                self.regs.set(rd, added);
                Ok(())
            }
            Ins::Qdsub { cond, rd, rm, rn } => {
                // The Q flag is not handled corerctly (or at all) here
                let doubled = self.regs.get(rd)?.saturating_mul(2);
                let added = self.regs.get(rm)?.saturating_sub(doubled);
                self.regs.set(rd, added);
                Ok(())
            }
            Ins::Qsax { cond, rd, rn, rm } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.1.saturating_sub(rn.0), rm.0.saturating_add(rn.1)),
                );
                Ok(())
            }
            Ins::Qsub { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                self.regs
                    .set(rd, self.regs.get(rn)?.saturating_sub(self.regs.get(rm)?));
                Ok(())
            }
            Ins::Qsub16 { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.0.saturating_sub(rn.0), rm.1.saturating_sub(rn.1)),
                );
                Ok(())
            }
            Ins::Qsub8 { cond, rd, rm, rn } => {
                // TODO: set Q flag when this calculation saturates
                fn to_bytes(w: u32) -> (i8, i8, i8, i8) {
                    (
                        (w & 0x0000_00ff) as i8,
                        (w >> 8) as i8,
                        (w >> 16) as i8,
                        (w >> 24) as i8,
                    )
                }
                let rm = to_bytes(self.regs.get(rm)?);
                let rn = to_bytes(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    (rm.0.saturating_sub(rm.0) as u32
                        | (rn.1.saturating_sub(rn.1) as u32) << 8
                        | (rn.2.saturating_sub(rn.2) as u32) << 16
                        | (rn.3.saturating_sub(rn.3) as u32) << 24),
                );
                Ok(())
            }
            Ins::Rev { cond, rd, rm } => {
                self.regs
                    .set(rd, u32::from_be_bytes(self.regs.get(rm)?.to_le_bytes()));
                Ok(())
            }
            Ins::Rev16 { cond, rd, rm } => {
                let src = self.regs.get(rm)?.to_le_bytes();
                self.regs
                    .set(rd, u32::from_le_bytes([src[1], src[0], src[3], src[2]]));
                Ok(())
            }
            Ins::Revsh { cond, rd, rm } => {
                let src = (self.regs.get(rm)? as u16).to_le_bytes();
                self.regs
                    .set(rd, i16::from_le_bytes([src[1], src[0]]) as i32 as u32);
                Ok(())
            }
            Ins::Rfe { .. } => unimplemented!(),
            Ins::Ror {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                self.regs.set(
                    rd,
                    self.regs
                        .shift_op(&self.regs.op2shift(op2)?, &unarm::ShiftOp::Ror, rd)?,
                );
                if *s {
                    todo!()
                }
                Ok(())
            }
            Ins::Rrx { .. } => unimplemented!(),
            Ins::Rsb {
                s,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = m.wrapping_sub(n);
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Rsc {
                s,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = m.wrapping_sub(n).wrapping_sub(self.regs.c() as u32);
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Sadd16 { cond, rd, rm, rn } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.0.saturating_add(rn.0), rm.1.saturating_add(rn.1)),
                );
                Ok(())
            }
            Ins::Sadd8 { cond, rd, rm, rn } => {
                fn to_bytes(w: u32) -> (i8, i8, i8, i8) {
                    (
                        (w & 0x0000_00ff) as i8,
                        (w >> 8) as i8,
                        (w >> 16) as i8,
                        (w >> 24) as i8,
                    )
                }
                let rm = to_bytes(self.regs.get(rm)?);
                let rn = to_bytes(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    (rm.0.saturating_add(rm.0) as u32
                        | (rn.1.saturating_add(rn.1) as u32) << 8
                        | (rn.2.saturating_add(rn.2) as u32) << 16
                        | (rn.3.saturating_add(rn.3) as u32) << 24),
                );
                Ok(())
            }
            Ins::Sasx { cond, rd, rn, rm } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(rm.1.saturating_add(rn.0), rm.0.saturating_sub(rn.1)),
                );
                Ok(())
            }
            Ins::Sbc {
                s,
                thumb,
                cond,
                rd,
                rn,
                op2,
            } => {
                let n = self.regs.get(rn)?;
                let m = self.regs.op2(op2)?;
                let result = n - m + self.regs.c() as u32;
                if *s {
                    todo!();
                }
                self.regs.set(rd, result);
                Ok(())
            }
            Ins::Sel { .. } => unimplemented!(),
            Ins::Sev { .. } => unimplemented!(),
            Ins::Setend { .. } => unimplemented!(),
            Ins::Shadd16 { cond, rd, rn, rm } => {
                let m = to_halfwords(self.regs.get(rm)?);
                let n = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords((m.0 >> 1) + (n.0 >> 1), (m.1 >> 1) + (n.1 >> 1)),
                );
                Ok(())
            }
            Ins::Shadd8 { cond, rd, rm, rn } => {
                let rm = to_bytes(self.regs.get(rm)?);
                let rn = to_bytes(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    (((rm.0 >> 1) + (rn.0 >> 1)) as u32)
                        | (((rm.1 >> 1) + (rn.1 >> 1)) as u32) << 8
                        | (((rm.2 >> 1) + (rn.2 >> 1)) as u32) << 16
                        | (((rm.3 >> 1) + (rn.3 >> 1)) as u32) << 24,
                );
                Ok(())
            }
            Ins::Shasx { cond, rd, rn, rm } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(
                        (rm.1 >> 1).saturating_add((rn.0 >> 1)),
                        (rm.0 >> 1).saturating_sub((rn.1 >> 1)),
                    ),
                );
                Ok(())
            }
            Ins::Shsax { cond, rd, rn, rm } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(
                        (rm.1 >> 1).saturating_sub((rn.0 >> 1)),
                        (rm.0 >> 1).saturating_add((rn.1 >> 1)),
                    ),
                );
                Ok(())
            }
            Ins::Shsub16 { cond, rd, rm, rn } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    from_halfwords(
                        (rm.0 >> 1).saturating_sub((rn.0 >> 1)),
                        (rm.1 >> 1).saturating_sub((rn.1 >> 1)),
                    ),
                );
                Ok(())
            }
            Ins::Shsub8 { cond, rd, rm, rn } => {
                let rm = to_bytes(self.regs.get(rm)?);
                let rn = to_bytes(self.regs.get(rn)?);
                self.regs.set(
                    rd,
                    ((rm.0 >> 1).saturating_sub((rm.0 >> 1)) as u32
                        | ((rn.1 >> 1).saturating_sub((rn.1 >> 1)) as u32) << 8
                        | ((rn.2 >> 1).saturating_sub((rn.2 >> 1)) as u32) << 16
                        | ((rn.3 >> 1).saturating_sub((rn.3 >> 1)) as u32) << 24),
                );
                Ok(())
            }
            Ins::Smla {
                cond,
                rd,
                rn,
                rn_side,
                rm,
                rm_side,
                ra,
            } => {
                let rm = to_halfwords(self.regs.get(rm)?);
                let rn = to_halfwords(self.regs.get(rn)?);
                let rm = match rm_side {
                    unarm::RegSide::Bottom => rm.0,
                    unarm::RegSide::Top => rm.1,
                };
                let rn = match rn_side {
                    unarm::RegSide::Bottom => rn.0,
                    unarm::RegSide::Top => rn.1,
                };

                let result = (rm as i32) * (rn as i32) + (self.regs.get(ra)? as i32);
                self.regs.set(rd, result as u32);
                Ok(())
            }
            Ins::Smlad { cond,rd, rn, rm, swap_rm, ra} => {
                let m = if *swap_rm {
                    let (left, right) = to_halfwords(self.regs.get(rm)?);
                    (right, left)
                } else {
                    to_halfwords(self.regs.get(rm)?)
                };
                let n = to_halfwords(self.regs.get(rn)?);
                let result = 
                (m.0 as i32 * n.0 as i32) +
                (m.1 as i32 * n.1 as i32) +
                (self.regs.get(ra)? as i32);
                self.regs.set(rd, result as u32);
                Ok(())
            }
        }
    }
}
