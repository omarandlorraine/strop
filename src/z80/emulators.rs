//! Module containing ways to emulate the Arm processor, in a way that's suitable for use with
//! strop.
use crate::z80::instruction_set::Z80Instruction;
use crate::Candidate;
use crate::Emulator;
use crate::Instruction;
use std::convert::TryInto;

use iz80::*;

/// A Z80 emulator
#[allow(missing_debug_implementations)]
pub struct Z80 {
    machine: PlainMachine,
    cpu: Cpu,
}

impl Default for Z80 {
    fn default() -> Self {
        Self {
            machine: PlainMachine::default(),
            cpu: Cpu::new_z80(),
        }
    }
}

impl<T: Instruction> Emulator<T> for Z80 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        let org: u16 = addr.try_into().unwrap();
        let encoding = candidate.encode();
        let end: u16 = (addr + encoding.len()).try_into().unwrap();

        // write the program into the CPU's memory
        for (offset, byte) in encoding.into_iter().enumerate() {
            self.machine.poke(org + offset as u16, byte);
        }

        self.cpu.registers().set_pc(org);

        for _ in 0..1000 {
            self.cpu.execute_instruction(&mut self.machine);
            let pc = self.cpu.registers().pc();
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}

impl Z80 {
    /// Runs a subroutine in the emulator.
    pub fn run_subroutine(&mut self, org: u16, from: u16, candidate: &Candidate<Z80Instruction>) {
        use rand::random;
        let encoding = candidate.encode();
        let end: u16 = org + encoding.len() as u16;

        let mut sp: u16 = random();
        while (org..end).contains(&sp) || (from..from + 3).contains(&sp) {
            sp = random();
        }
        let sp = sp;

        // write the subroutine into the CPU's memory
        for (offset, byte) in encoding.into_iter().enumerate() {
            self.machine.poke(org + offset as u16, byte);
        }

        // write a CALL instruction into the CPU's memory
        let org_bytes = org.to_le_bytes();
        for (offset, byte) in vec![0xcd, org_bytes[0], org_bytes[1]]
            .into_iter()
            .enumerate()
        {
            self.machine.poke(from + offset as u16, byte);
        }

        // execute the CALL instruction
        self.cpu.registers().set_pc(from);
        self.cpu.execute_instruction(&mut self.machine);
        assert_eq!(self.cpu.registers().pc(), org);

        for _ in 0..1000 {
            self.cpu.execute_instruction(&mut self.machine);

            let pc = self.cpu.registers().pc();

            // So the PC is pointing to the instruction right after the CALL instruction
            if pc == from + 3 {
                break;
            }

            // So the stack pointer has returned to where it was before the CALL.
            if self.cpu.registers().get16(Reg16::SP) == sp {
                break;
            }
        }
    }
}

impl Z80 {
    /// Returns the 32-bit value represented by the emulated CPU's D, E, H and L registers.
    pub fn get_dehl(&self) -> u32 {
        u32::from_le_bytes([
            self.cpu.immutable_registers().get8(iz80::Reg8::D),
            self.cpu.immutable_registers().get8(iz80::Reg8::E),
            self.cpu.immutable_registers().get8(iz80::Reg8::H),
            self.cpu.immutable_registers().get8(iz80::Reg8::L),
        ])
    }

    /// Writes a 32-bit value to the emulated CPU's D, E, H and L registers.
    pub fn set_dehl(&mut self, val: u32) {
        let bytes = val.to_le_bytes();
        self.cpu.registers().set8(iz80::Reg8::D, bytes[3]);
        self.cpu.registers().set8(iz80::Reg8::E, bytes[2]);
        self.cpu.registers().set8(iz80::Reg8::H, bytes[1]);
        self.cpu.registers().set8(iz80::Reg8::L, bytes[0]);
    }

    pub fn set_sp(&mut self, val: u16) {
        self.cpu.registers().set16(Reg16::SP, val);
    }

    pub fn get_sp(&mut self) -> u16 {
        self.cpu.registers().get16(Reg16::SP)
    }

    pub fn get_pc(&mut self) -> u16 {
        self.cpu.registers().pc()
    }
}

/// An Intel 8080 emulator
#[allow(missing_debug_implementations)]
pub struct I8080 {
    machine: PlainMachine,
    cpu: Cpu,
}

impl Default for I8080 {
    fn default() -> Self {
        Self {
            machine: PlainMachine::default(),
            cpu: Cpu::new_8080(),
        }
    }
}

impl<T: Instruction> Emulator<T> for I8080 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        let org: u16 = addr.try_into().unwrap();
        let encoding = candidate.encode();
        let end: u16 = (addr + encoding.len()).try_into().unwrap();

        // write the program into the CPU's memory
        for (offset, byte) in encoding.into_iter().enumerate() {
            self.machine.poke(org + offset as u16, byte);
        }

        self.cpu.registers().set_pc(org);

        for _ in 0..1000 {
            self.cpu.execute_instruction(&mut self.machine);
            let pc = self.cpu.registers().pc();
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}
