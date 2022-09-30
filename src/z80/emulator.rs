use crate::emulator::Emulator;
/// Module implementing the
use rustzx_z80::{Z80Bus, Z80};
use std::convert::TryInto;

pub struct BusZ80 {
    memory: [u8; 65536],
}

impl Z80Bus for BusZ80 {
    fn read_internal(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_internal(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn wait_mreq(&mut self, _addr: u16, _clk: usize) {}

    fn wait_no_mreq(&mut self, _addr: u16, _clk: usize) {}

    fn wait_internal(&mut self, _clk: usize) {}

    fn read_io(&mut self, _port: u16) -> u8 {
        panic!()
    }

    fn write_io(&mut self, _port: u16, _data: u8) {
        panic!()
    }

    fn read_interrupt(&mut self) -> u8 {
        0
    }

    fn reti(&mut self) {}

    fn halt(&mut self, _halted: bool) {}

    fn int_active(&self) -> bool {
        false
    }

    fn nmi_active(&self) -> bool {
        false
    }

    fn pc_callback(&mut self, _addr: u16) {}
}

pub struct EmulatorZ80 {
    cpu: Z80,
    bus: BusZ80,
}

impl Emulator for EmulatorZ80 {
    fn run(&mut self, org: usize, budget: u32, bytes: &mut dyn Iterator<Item = u8>) {
        // the emulator uses 0xff as a sentinel to end the currently running program.
        self.load(org, bytes);
        self.cpu.regs.set_pc(org.try_into().unwrap());
        for _ in 0..budget {
            self.cpu.emulate(&mut self.bus);
        }
    }

    fn load(&mut self, org: usize, bytes: &mut dyn Iterator<Item = u8>) {
        let mut addr: u16 = org.try_into().unwrap();
        for b in bytes {
            self.bus.write_internal(addr, b);
            addr += 1;
        }
    }
}