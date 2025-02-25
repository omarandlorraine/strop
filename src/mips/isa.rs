//! Module representing MIPS I instruction set architecture

use crate::Encode;
use crate::Step;
use trapezoid_core::cpu::Instruction;

/// Represents a MIPS instruction
#[derive(Clone, PartialEq)]
pub struct Insn(u32);

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.decode())
    }
}

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} 0x{:08x}", self.decode(), self.0)
    }
}

impl crate::subroutine::ShouldReturn for Insn {
    fn should_return(&self) -> Option<crate::StaticAnalysis<Self>> {
                if *self == Self::jr_ra() {
            return None;
        }
        Some(crate::StaticAnalysis::<Self> {
            advance: Self::make_return,
            offset: 0,
            reason: "ShouldReturn",
        })

    }
}

impl Insn {
    fn decode(&self) -> Instruction {
        Instruction::from_u32(self.0, 0)
    }

    /// Returns a `jr $ra` instruction, which is what's used to return from subroutines.
    pub fn jr_ra() -> Self {
        Self(0x03E00008)
    }

        fn make_return(&mut self) -> crate::IterationResult {
        // TODO: There are other possible return instructions here.
        use std::cmp::Ordering;

        match self.0.cmp(&Self::jr_ra().0) {
            Ordering::Less => {
                *self = Self::jr_ra();
                Ok(())
            }
            Ordering::Greater => Err(crate::StepError::End),
            Ordering::Equal => unreachable!(),
        }
    }


    /// Called after a mutation; this ensures that the u32 member encodes an actually valid MIPS
    /// instruction
    fn fixup(&mut self) {
        // TODO: this could potentially overflow.
        use trapezoid_core::cpu::Opcode;
        loop {
            if self.0 & 0xfc000000 == 0 {
                // This is an R format instruction. If the opcode is invalid, then to fix that we need
                // to increment the instruction word by 1.
                if self.decode().opcode == Opcode::Invalid {
                    self.0 += 1;
                } else {
                    break;
                }
            } else {
                // Could be an I or J format instruction. If the opcode is invalid, then to fix
                // that we need to increment the instruction word by 0x0400_0000.
                if self.decode().opcode == Opcode::Invalid {
                    self.0 += 0x0400_0000;
                } else {
                    break;
                }
            }
        }
    }

    /// Skip to the next opcode (this increments either the `funct` field or the `opcode` field as
    /// appropriate)
    pub fn next_opcode(&mut self) -> bool {
        if self.0 >= 0xefff_ffff {
            return false;
        }
        if self.0 & 0xfc000000 == 0 {
            // It's an R format instruction so to go to the next opcode we need to increment by 1
            self.0 += 1;
            self.fixup();
        } else {
            // It's an I or J format instruction. To go to the next opcode, add 0x0400_0000.
            self.0 += 0x0400_0000;
            self.fixup();
        }
        true
    }
}

impl crate::Disassemble for Insn {
    fn dasm(&self) {
        println!("{:?}", self.0);
    }
}

impl Step for Insn {
    fn first() -> Self {
        Self(0)
    }

    fn next(&mut self) -> crate::IterationResult {
        if self.0 >= 0xefff_ffff {
            // There are no valid instructions in this range.
            Err(crate::StepError::End)
        } else {
            self.0 += 1;
            self.fixup();
            Ok(())
        }
    }
}

impl Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod test {

    #[test]
    #[ignore]
    fn can_iterate_over_all_instructions() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn::first();

        while i.next().is_ok() {
            assert_ne!(format!("{}", i), "Invalid instruction", "{:08x}", i.0);
        }
    }

    #[test]
    fn can_iterate_over_the_first_few_instructions() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn::first();

        for _ in 0..0xffff {
            assert!(i.next().is_ok());
            assert_ne!(format!("{}", i), "Invalid instruction", "{:08x}", i.0);
        }
    }

    #[test]
    fn can_iterate_until_the_end() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn(0xefff_ff00);

        while i.next().is_ok() {
            assert_ne!(format!("{}", i), "Invalid instruction", "{:08x}", i.0);
        }
    }

    #[test]
    fn jr_ra() {
        use super::Insn;
        use trapezoid_core::cpu::Opcode;
        use trapezoid_core::cpu::RegisterType;

        let i = Insn::jr_ra();
        let d = i.decode();
        assert_eq!(d.opcode, Opcode::Jr);
        assert_eq!(d.imm5(), 0);
        assert_eq!(d.rt(), RegisterType::Zero);
        assert_eq!(d.rd(), RegisterType::Zero);
        assert_eq!(d.rs(), RegisterType::Ra);
    }
}
