//! Module containing emulators for the 6502, with the necessary additions making them suitable for
//! use with strop.
use crate::Candidate;
use crate::Emulator;
use crate::Instruction;
use std::convert::TryInto;

use mos6502::instruction::Nmos6502;
use mos6502::memory::Memory;

/// This emulates a basic MOS 6502. Extras like the illegal instructions, CMOS instructions, etc, are
/// not supported.
#[derive(Debug)]
pub struct Mos6502 {
    cpu: mos6502::cpu::CPU<Memory, Nmos6502>,
}

impl Default for Mos6502 {
    fn default() -> Self {
        let mut cpu = mos6502::cpu::CPU::new(Memory::new(), Nmos6502);
        cpu.registers.accumulator = 0;
        cpu.registers.index_x = 0;
        cpu.registers.index_y = 0;
        Self { cpu }
    }
}

impl Mos6502 {
    /// return value of accumulator
    pub fn a(&self) -> u8 {
        self.cpu.registers.accumulator
    }
}

impl<T: Instruction> Emulator<T> for Mos6502 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        use mos6502::memory::Bus;
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
