//! Module containing ways to emulate the Arm processor, in a way that's suitable for use with
//! strop.
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
