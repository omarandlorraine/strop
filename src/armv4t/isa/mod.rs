//! Module for representing ARMv4T machine code instructions.

pub mod decode;
mod mutate;
use crate::static_analysis::Fixup;
use crate::StaticAnalysis;
use crate::{Step, StepError};

/// Checks whether an immediate value is encoded in the canonical way.
///
/// Some immediate values have more than one encoding, so this function culls a lot of these
/// duplicates. (the result: the brute-force search is culled).
fn canonical_immediate(encoding: u32) -> bool {
    let shift = encoding >> 8 & 0x0f;
    let value = (encoding & 0xff) as u8;

    if value == 0 {
        // doesn't matter what shift is if value is 0.
        return shift == 0;
    }

    if value & 0x03 == 0 {
        // the value could be shifted over in itself and the shift value could compensate
        return false;
    }

    true
}

/// Represents an ARMv4T machine code instruction.
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Insn(pub(crate) u32);

impl Insn {
    /// Return the instruction, `bx lr`.
    pub fn bx_lr() -> Self {
        Self(0xe12fff1e)
    }

    /// Returns the instruction for popping the registers off the stack
    pub fn pop(r: &[crate::armv4t::isa::decode::Register]) -> Self {
        use crate::armv4t::isa::decode::Register;
        let mut i = 0xe8bd0000u32;
        for reg in [
            Register::R0,
            Register::R1,
            Register::R2,
            Register::R3,
            Register::R4,
            Register::R5,
            Register::R6,
            Register::R7,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
            Register::R12,
            Register::Lr,
            Register::Sp,
            Register::Pc,
        ]
        .iter()
        .enumerate()
        {
            if r.contains(reg.1) {
                i |= 1 << (reg.0 as u32);
            }
        }
        Self(i)
    }

    /// Returns the instruction for pushing the registers onto the stack
    pub fn push(r: &[crate::armv4t::isa::decode::Register]) -> Self {
        use crate::armv4t::isa::decode::Register;
        let mut i = 0xe92d0000u32;
        for reg in [
            Register::R0,
            Register::R1,
            Register::R2,
            Register::R3,
            Register::R4,
            Register::R5,
            Register::R6,
            Register::R7,
            Register::R8,
            Register::R9,
            Register::R10,
            Register::R11,
            Register::R12,
            Register::Lr,
            Register::Sp,
            Register::Pc,
        ]
        .iter()
        .enumerate()
        {
            if r.contains(reg.1) {
                i |= 1 << (reg.0 as u32);
            }
        }
        Self(i)
    }

    /// Makes sure that the instruction is a valid one. If it does not encode a valid instruction
    /// then this method returns a Fixup rectifying the problem
    pub fn fixup(&mut self) -> crate::StaticAnalysis<Self> {
        // TODO: PSR instructions shouldn't ever take PC or SP or LR as their argument
        use crate::static_analysis::Fixup;
        use unarm::arm::Opcode;

        // Don't bother with synchronisation primitives
        Fixup::check(
            !matches!(self.decode().op, Opcode::Strex | Opcode::Swp | Opcode::Ldrex | Opcode::Strexd | Opcode::Ldrexd | Opcode::Ldrexb | Opcode::Ldrexh),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // Don't bother generating `bkpt` instructions
        Fixup::check(
            !matches!(self.decode().op, Opcode::Bkpt),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // For instructions with an immediate operand, check that the instruction has the canonical
        // encoding.
        if self.0 & 0x0e00_0000 == 0x0200_0000 {
            Fixup::check(
                canonical_immediate(self.0 & 0xfff),
                "UncanonicalImmediateEncoding",
                Self::increment,
                0,
            )?;
        }

        // For store-halfword instructions, the `U` bit is a don't-care if `W == 0`. So skip these
        // instructions if `U` == 1 and `W` == 0.
        Fixup::check(
            !(matches!(
                self.decode().op,
                Opcode::Str
                    | Opcode::Ldr
                    | Opcode::StrT
                    | Opcode::LdrT
                    | Opcode::StrB
                    | Opcode::LdrB
                    | Opcode::StrBt
                    | Opcode::LdrBt
                    | Opcode::StrH
                    | Opcode::LdrH
                    | Opcode::LdrSb
                    | Opcode::LdrSh
            ) && self.0 & 0x00a0_0090 != 0x0080_0090),
            "InvalidEncoding",
            Self::increment,
            0,
        )?;

        // coprocessors are not supported, skip these instructions
        Fixup::check(
            !matches!(self.decode().op, Opcode::Stc | Opcode::Stc2 |Opcode::Mrrc |Opcode::Mrc | Opcode::Ldc | Opcode::Mcrr | Opcode::Cdp |Opcode::Mcr),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // Some instructions don't get introduced until after ARMv4T
        // TODO: There's more to exclude here.
        Fixup::check(
            !matches!(
                self.decode().op,
                Opcode::Ssat
                    | Opcode::Ssat16
                    | Opcode::Sxtb
                    | Opcode::Sxtab
                    | Opcode::Pkhtb
                    | Opcode::Pkhbt
                    | Opcode::Sel
                    | Opcode::Sxtah
                    | Opcode::Sxth
                    | Opcode::Rev
                    | Opcode::Rev16
                    | Opcode::Uxtab
                    | Opcode::Uxtab16
                    | Opcode::Uxtb
                    | Opcode::Uxtb16
                    | Opcode::Usat
                    | Opcode::Usat16
                    | Opcode::Uxtah
                    | Opcode::Uxth
                    | Opcode::Revsh
                    | Opcode::Smlad
                    | Opcode::Smlsd
                    | Opcode::Smuad
                    | Opcode::Smusd
                    | Opcode::Smlald
                    | Opcode::Smlsld
                    | Opcode::Smmla
                    | Opcode::Smmls
                    | Opcode::Smmul
                    | Opcode::Usada8
                    | Opcode::Usad8
                    | Opcode::Sadd16
                    | Opcode::Sasx
                    | Opcode::Ssax
                    | Opcode::Ssub16
                    | Opcode::Sadd8
                    | Opcode::Ssub8
                    | Opcode::Qadd16
                    | Opcode::Qasx
                    | Opcode::Qsax
                    | Opcode::Qsub16
                    | Opcode::Qadd8
                    | Opcode::Qsub8
                    | Opcode::Shadd16
                    | Opcode::Shasx
                    | Opcode::Shsax
                    | Opcode::Shsub16
                    | Opcode::Shadd8
                    | Opcode::Shsub8
                    | Opcode::Uadd16
                    | Opcode::Uasx
                    | Opcode::Usax
                    | Opcode::Usub16
                    | Opcode::Uadd8
                    | Opcode::Usub8
                    | Opcode::Uqadd16
                    | Opcode::Uqasx
                    | Opcode::Uqsax
                    | Opcode::Uqsub16
                    | Opcode::Uqadd8
                    | Opcode::Uqsub8
                    | Opcode::Uhadd16
                    | Opcode::Uhasx
                    | Opcode::Uhsax
                    | Opcode::Uhsub16
                    | Opcode::Uhsub8
                    | Opcode::Sxtab16
                    | Opcode::Sxtb16
                    | Opcode::Uhadd8
            ),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // Don't generate `SWI` instructions
        Fixup::check(
            !matches!(self.decode().op, Opcode::Svc),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // I think unarm disassembles these incorrectly
        Fixup::check(
            !(matches!(self.decode().op, Opcode::B | Opcode::Bl) && self.0 & 0x00c0_0000 != 0),
            "UnsupportedInstruction",
            Self::increment,
            0,
        )?;

        // Don't generate any illegal instructions.
        Fixup::check(
            !matches!(self.decode().op, Opcode::Illegal | Opcode::Udf),
            "IllegalInstruction",
            Self::increment,
            0,
        )?;
        Fixup::check(
            self.decode().parse(&Default::default()).mnemonic != "<illegal>",
            "IllegalInstruction",
            Self::increment,
            0,
        )?;

        Ok(())
    }

    /// Increments the isntruction word by 1
    pub fn increment(&mut self) -> crate::IterationResult {
        if self.0 > 0xf0000000 {
            Err(StepError::End)
        } else {
            self.0 += 1;
            Ok(())
        }
    }

    /// Decodes the instruction
    pub fn decode(&self) -> unarm::arm::Ins {
        unarm::arm::Ins::new(self.0, &Default::default())
    }

    /// Skips to the "horrid nybble". ignores the bottom four bits, since for all instruyctions,
    /// decoding that is trivial. It's for the Horrid Nybble that things get messy.
    pub fn next_horrid_nybble(&mut self) -> crate::IterationResult {
        self.0 |= 0x0000_000f;
        self.next()
    }

    /// Skips all other instructions having the same condition code.
    pub fn next_condition(&mut self) -> crate::IterationResult {
        self.0 |= 0x0fff_ffff;
        self.next()
    }

    /// Skips to the "next opcode". ignores the fields like `Rn`, and `Rm`, the register lists and
    /// offsets and things, in the hope of hitting on the next instruction. This method won't hit
    /// on the branch exchange instruction.
    pub fn next_opcode(&mut self) -> crate::IterationResult {
        self.0 |= 0x000f_ffff;
        self.next()
    }

    fn make_return(&mut self) -> crate::IterationResult {
        // TODO: There are other possible return instructions here.
        use std::cmp::Ordering;

        match self.0.cmp(&Self::bx_lr().0) {
            Ordering::Less => {
                *self = Self::bx_lr();
                Ok(())
            }
            Ordering::Greater => Err(StepError::End),
            Ordering::Equal => unreachable!(),
        }
    }
}

impl Step for Insn {
    fn first() -> Self {
        Insn(0)
    }

    fn next(&mut self) -> crate::IterationResult {
        self.increment()?;
        while let Err(e) = self.fixup() {
            (e.advance)(self)?;
        }
        Ok(())
    }
}

impl crate::subroutine::ShouldReturn for Insn {
    fn should_return(&self, offset: usize) -> StaticAnalysis<Self> {
        if *self == Self::bx_lr() {
            Ok(())
        } else {
            Fixup::err("ShouldReturn", Self::make_return, offset)
        }
    }

    fn allowed_in_subroutine(&self, offset: usize) -> crate::StaticAnalysis<Self> {
        use crate::armv4t::data::Register;
        use crate::dataflow::DataFlow;
        Fixup::check(
            !(self.reads(&Register::Sp)
                || self.writes(&Register::Sp)
                || self.reads(&Register::Lr)
                || self.writes(&Register::Lr)
                || self.reads(&Register::Pc)
                || self.writes(&Register::Pc)),
            "LeaveSpLrAndPcAlone",
            Self::next,
            offset,
        )
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl crate::Encode<u32> for Insn {
    fn encode(&self) -> Vec<u32> {
        vec![self.0]
    }
}

impl crate::Disassemble for Insn {
    fn dasm(&self) {
        println!("\t{:?}", self);
    }
}

impl crate::Branch for Insn {}

#[cfg(test)]
mod test {
    use super::Insn;
    use crate::Step;

    fn emulator_knows_it(i9n: super::Insn) -> bool {
        use crate::Encode;
        use armv4t_emu::{reg, Cpu, ExampleMem, Mode};
        let mut mem = ExampleMem::new_with_data(&i9n.encode());
        let mut cpu = Cpu::new();
        cpu.reg_set(Mode::User, reg::PC, 0x00);
        cpu.reg_set(Mode::User, reg::CPSR, 0x10);
        cpu.step(&mut mem)
    }

    #[test]
    fn bx_lr() {
        assert_eq!("bx lr", &format!("{}", super::Insn::bx_lr()));
    }

    #[test]
    fn should_skip() {
        use super::Insn;

        // If it disassembles as `<illegal>` then the fixup method should fix it up!
        assert!(
            Insn(0xe01001bb).fixup().is_err(),
            "{:?}",
            Insn(0xe01001bb).decode().parse(&Default::default())
        );
    }

    #[test]
    fn should_return() {
        use crate::subroutine::ShouldReturn;

        // get the first instruction which decodes to `andeq r0, r0, r0` or whatever
        let mut i = super::Insn::first();

        // this should return a static analysis that changes it to `bx lr`
        let sa = i.should_return(0).err().unwrap();

        // so advance it.
        (sa.advance)(&mut i).unwrap();
        assert_eq!(i, super::Insn::bx_lr());

        // this time it should not return a static analysis
        assert!(i.should_return(0).is_ok());

        // but if we advance to some other instruction, ...
        i.next().unwrap();

        // ... then this should return a static analysis that goes to an error
        let sa = i.should_return(0).err().unwrap();
        assert!((sa.advance)(&mut i).is_err());
    }

    fn check(i: &super::Insn) {
        use crate::armv4t::data::Register;
        use crate::dataflow::DataFlow;

        assert!(emulator_knows_it(*i), "{i:?}");
        assert!(!format!("{:?}", i).contains("illegal"), "{:?}", i);

        if format!("{i}").contains("r4") {
            assert!(
                i.reads(&Register::R4) || i.writes(&Register::R4),
                "{i:?} doesn't read or write R4. {:?}",
                i.decode().defs(&Default::default())
            );
        }
    }

    #[test]
    fn regressions() {
        // All these instructions are unsupported, and therefore the .fixup() method should
        // change them.

        // unpredictable
        assert!(Insn(0x00001094).fixup().is_err());

        // coprocessor instructions
        assert!(Insn(0x1c200000).fixup().is_err());
        assert!(Insn(0xec300000).fixup().is_err());
        assert!(Insn(0xec400000).fixup().is_err());
        assert!(Insn(0xee000000).fixup().is_err());
        assert!(Insn(0xee4bdd77).fixup().is_err());
    }

    #[test]
    #[ignore]
    fn all_instructions() {
        use crate::Step;

        let mut i = Insn::first();
        while i.next().is_ok() {
            check(&i);
        }
    }

    #[test]
    #[ignore]
    fn duplicate_instructions() {
        let mut i = Insn(0xe000_0000);
        i.next().unwrap();
        'outer: loop {
            for ignore in [
                "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "lr",
                "sp", "pc",
            ] {
                if format!("{i}").contains(ignore) {
                    i.next_horrid_nybble().unwrap();
                    continue 'outer;
                }
            }
            let mut j = i;

            println!("{i:?}");

            while j.next_horrid_nybble().is_ok() {
                assert_ne!(format!("{i}"), format!("{j}"), "\n{i:?}\n{j:?}");
            }
            if i.next_horrid_nybble().is_err() {
                break;
            }
        }
    }

    #[test]
    #[ignore]
    fn noncanonical_immediates() {
        //let mut i = Insn(0xe000_0000);
        let mut i = Insn(0xe2000201);
        loop {
            if i.0 & 0x0e00_0000 != 0x0200_0ea1 {
                break;
            }
            for ignore in [
                "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "lr",
                "sp", "pc",
            ] {
                while format!("{i}").contains(ignore) {
                    i.next_horrid_nybble().unwrap();
                }
            }
            let mut j = i;

            println!("{i:?}");

            while j.next().is_ok() {
                if j.0 & 0xffff_f000 != i.0 & 0xffff_f000 {
                    break;
                }
                assert_ne!(format!("{i}"), format!("{j}"), "\n{i:?}\n{j:?}");
            }
            if i.next().is_err() {
                break;
            }
        }
    }
}
