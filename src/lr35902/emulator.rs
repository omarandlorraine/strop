use crate::emulator::Emulator;
use lr35902::cpu::cpu::Cpu;

extern crate lr35902;

pub struct EmulatorLR35902 {
    cpu: Cpu,
}

impl Emulator for EmulatorLR35902 {
    fn run(&mut self, org: usize, _budget: u32, bytes: &mut dyn Iterator<Item = u8>) {
        self.load(org, bytes);
        self.cpu.registers.pc = org as u16;

    }

    fn load(&mut self, org: usize, bytes: &mut dyn Iterator<Item = u8>) {
    }
}
