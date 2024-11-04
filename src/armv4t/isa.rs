//! Module for representing ARMv4T machine code instructions.

pub mod decode;

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
}

impl crate::Iterable for Insn {
    fn first() -> Self {
        Insn(0)
    }

    fn step(&mut self) -> bool {
        if self.0 > 0xfffffffe {
            false
        } else {
            self.0 += 1;
            true
        }
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
        println!("{:?}", self);
    }
}

#[cfg(test)]
mod test {

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
    #[ignore]
    fn all_instructions() {
        use crate::Iterable;

        let mut i = super::Insn(0xff7affff);
        // let mut i = super::Insn::first();

        while i.step() {
            // check that the instruction can be disassembled
            assert_eq!(format!("{:?}", i).len(), 95, "{:?}", i);

            // println!("{:?}", i);

            assert!(!format!("{:?}", i).contains("illegal"), "{:?}", i);

            // check that the emulator can execute the instruction
            if !emulator_knows_it(i) {
                let beginning = i;
                let mut end = i;
                while !emulator_knows_it(i) {
                    end = i;
                    println!("the emulator can't run {:?}", i);
                    i.step();
                }
                println!("the range is {:?}..{:?} inclusive", beginning.0, end.0);
                panic!("found a range of instructions visited by the .increment method that the emulator doesn't know");
            }
        }
    }
}
