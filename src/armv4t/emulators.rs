//! Module containing ways to emulate the Arm processor, in a way that's suitable for use with
//! strop.
use crate::Candidate;
use crate::Emulator;
use crate::Instruction;
use std::collections::BTreeMap;
use std::convert::TryInto;

/// This emulates an ArmV4T type processor.
#[derive(Debug, Default)]
pub struct ArmV4T {
    cpu: armv4t_emu::Cpu,
    mem: ArmMemory,
}

#[derive(Debug, Default)]
struct ArmMemory(BTreeMap<u32, u8>);

impl ArmMemory {
    /// Constructs a new ArmMemory from the provided slice. Data is copied
    /// contiguously from the slice into address [0..data.len()]
    pub fn insert(&mut self, address: u32, data: &[u8]) {
        for (i, b) in data.iter().cloned().enumerate() {
            self.0.insert(address + i as u32, b);
        }
    }
}

impl armv4t_emu::Memory for ArmMemory {
    fn r8(&mut self, addr: u32) -> u8 {
        *self.0.get(&addr).unwrap_or(&0)
    }

    fn r16(&mut self, addr: u32) -> u16 {
        self.r8(addr) as u16 | (self.r8(addr + 1) as u16) << 8
    }

    fn r32(&mut self, addr: u32) -> u32 {
        self.r16(addr) as u32 | (self.r16(addr + 2) as u32) << 16
    }

    fn w8(&mut self, addr: u32, val: u8) {
        self.0.insert(addr, val);
    }

    fn w16(&mut self, addr: u32, val: u16) {
        self.w8(addr, val as u8);
        self.w8(addr + 1, (val >> 8) as u8);
    }

    fn w32(&mut self, addr: u32, val: u32) {
        self.w16(addr, val as u16);
        self.w16(addr + 2, (val >> 16) as u16);
    }
}

impl<T: Instruction> Emulator<T> for ArmV4T {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        use armv4t_emu::{reg, Mode};

        let org: u32 = addr.try_into().unwrap();
        let encoding = candidate.encode();
        let end: u32 = (addr + encoding.len()).try_into().unwrap();

        self.mem.insert(org, &encoding);

        self.cpu.reg_set(Mode::User, reg::PC, org);
        self.cpu.reg_set(Mode::User, reg::CPSR, 0x20); // go into thumb mode
        assert!(self.cpu.thumb_mode());

        for _ in 0..1000 {
            self.cpu.step(&mut self.mem);
            let pc = self.cpu.reg_get(Mode::User, reg::PC);
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}

#[cfg(test)]
use crate::armv4t::instruction_set::Thumb;

#[cfg(test)]
impl ArmV4T {
    pub fn run_thumb(&mut self, instruction: &Thumb) -> bool {
        use armv4t_emu::{reg, Mode};

        let program = Candidate::new(vec![*instruction]);

        let org: u32 = 0x8000;
        let encoding = program.encode();

        self.mem.insert(org, &encoding);

        self.cpu.reg_set(Mode::User, reg::PC, org);
        self.cpu.reg_set(Mode::User, reg::CPSR, 0x20); // go into thumb mode

        assert!(self.cpu.thumb_mode(), "The CPU didn't go into Thumb mode");

        self.cpu.step(&mut self.mem)
    }

    pub fn thumb_mode(&self) -> bool {
        self.cpu.thumb_mode()
    }
}
