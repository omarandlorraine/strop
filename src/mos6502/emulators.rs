//! Module containing emulators for the 6502, with the necessary additions making them suitable for
//! use with strop.
use crate::Candidate;
use crate::Emulator;
use crate::Instruction;
use std::convert::TryInto;

/// This emulates a basic MOS 6502. Extras like the illegal instructions, CMOS instructions, etc, are
/// not supported.
#[derive(Debug, Default)]
pub struct Mos6502 {
    cpu: mos6502::cpu::CPU,
}

impl Mos6502 {
    /// return value of accumulator
    pub fn a(&self) -> u8 {
        self.cpu.registers.accumulator as u8
    }
}

impl<T: Instruction> Emulator<T> for Mos6502 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        let org: u16 = addr.try_into().unwrap();
        let encoding = candidate.encode();
        let end: u16 = (addr + encoding.len()).try_into().unwrap();

        self.cpu.memory.set_bytes(org, &encoding);
        self.cpu.registers.program_counter = org;

        for _ in 0..1000 {
            self.cpu.single_step();
            let pc = self.cpu.registers.program_counter;
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}
