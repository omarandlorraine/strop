//! Module containing emulators for the 6502, with the necessary additions making them suitable for
//! use with strop.
use crate::StropError;
use crate::m6502::Insn;
use mos6502::memory::Memory;
use std::convert::TryInto;

/// This emulates one of the 6502 CPU variants
#[derive(Debug)]
pub struct Emulator<V: mos6502::Variant> {
    cpu: mos6502::cpu::CPU<Memory, V>,
}

impl<V: mos6502::Variant + Default> Default for Emulator<V> {
    fn default() -> Self {
        let mut cpu = mos6502::cpu::CPU::new(Memory::new(), V::default());
        cpu.registers.accumulator = 0;
        cpu.registers.index_x = 0;
        cpu.registers.index_y = 0;
        Self { cpu }
    }
}

impl<V: mos6502::Variant> Emulator<V> {
    /// return value of accumulator
    pub fn get_a(&self) -> u8 {
        self.cpu.registers.accumulator
    }

    /// sets accumulator
    pub fn set_a(&mut self, val: u8) {
        self.cpu.registers.accumulator = val;
    }

    /// return value of accumulator
    pub fn get_x(&self) -> u8 {
        self.cpu.registers.index_x
    }

    /// sets accumulator
    pub fn set_x(&mut self, val: u8) {
        self.cpu.registers.index_x = val;
    }

    /// return the sixteen bits value held in the A and X registers
    pub fn get_ax(&self) -> u16 {
        u16::from_le_bytes([self.get_a(), self.get_x()])
    }

    /// return the sixteen bits value held in the A and X registers
    pub fn set_ax(&mut self, val: u16) {
        let [a, x] = u16::to_le_bytes(val);
        self.set_a(a);
        self.set_x(x);
    }
}

impl<V: Clone + mos6502::Variant> Emulator<V> {
    /// Runs a subroutine in emulation
    pub fn run(&mut self, program: &crate::Sequence<Insn<V>>) -> Result<(), StropError> {
        use crate::Encode;
        use mos6502::memory::Bus;
        let addr = 0x300;
        let org: u16 = addr.try_into().unwrap();
        let encoding = program.encode();
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
        Ok(())
    }
}
