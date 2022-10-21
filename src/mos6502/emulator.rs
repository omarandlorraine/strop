use crate::emulator::Emulator;
use mos6502::cpu::CPU;

#[derive(Debug)]
pub struct Emulator6502 {
    cpu: CPU,
}

impl Emulator6502 {
    pub fn get_a(&mut self) -> u8 {
        u8::from_ne_bytes(self.cpu.registers.accumulator.to_ne_bytes())
    }

    pub fn set_a(&mut self, val: u8) {
        self.cpu.registers.accumulator = i8::from_ne_bytes(val.to_ne_bytes());
    }
}

impl Default for Emulator6502 {
    fn default() -> Self {
        Self { cpu: CPU::new() }
    }
}

impl Emulator for Emulator6502 {
    fn run(&mut self, org: usize, _budget: u32, bytes: &mut dyn Iterator<Item = u8>) {
        // the emulator uses 0xff as a sentinel to end the currently running program.
        let prog = &bytes.chain(vec![0xff]).collect::<Vec<_>>();
        self.cpu.memory.set_bytes(org as u16, prog);

        self.cpu.registers.program_counter = (org as u16);
        self.cpu.run();
    }

    fn load(&mut self, org: usize, bytes: &mut dyn Iterator<Item = u8>) {
        self.cpu
            .memory
            .set_bytes(org as u16, &bytes.collect::<Vec<_>>());
    }
}
