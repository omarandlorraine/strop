//! Module containing ways to emulate the Arm processor, in a way that's suitable for use with
//! strop.
use crate::armv4t::Insn;
use crate::Sequence;
use crate::StropError;
use std::collections::BTreeMap;

/// This emulates an ARMv4T type processor.
#[derive(Debug, Default)]
pub struct Emulator {
    cpu: armv4t_emu::Cpu,
    mem: ArmMemory,
}

impl Emulator {
    /// sets R0 to the given value
    pub fn set_r0(&mut self, a: i32) {
        use armv4t_emu::Mode;
        self.cpu.reg_set(Mode::User, 0, a as u32)
    }

    /// returns the value of R0
    pub fn get_r0(&self) -> i32 {
        use armv4t_emu::Mode;
        self.cpu.reg_get(Mode::User, 0) as i32
    }

    /// sets R0 to the given value
    pub fn set_r1(&mut self, a: i32) {
        use armv4t_emu::Mode;
        self.cpu.reg_set(Mode::User, 1, a as u32)
    }

    /// returns the value of R1
    pub fn get_r1(&self) -> i32 {
        use armv4t_emu::Mode;
        self.cpu.reg_get(Mode::User, 1) as i32
    }
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

impl Emulator {
    /// Runs a sequence of instructions in the emulator
    pub fn run(&mut self, candidate: &Sequence<Insn>) -> Result<(), StropError> {
        use crate::Encode;
        use armv4t_emu::{reg, Mode};

        let org: u32 = 0;
        let encoding = candidate.encode();
        let end: u32 = org + encoding.len() as u32;

        self.mem.insert(org, &encoding);

        // Start at 0, with a stack pointer, and in thumb mode
        self.cpu.reg_set(Mode::User, reg::PC, org);
        self.cpu.reg_set(Mode::User, reg::SP, 0x200);
        self.cpu.reg_set(Mode::User, reg::CPSR, 0x20);

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
        Ok(())
    }
}
