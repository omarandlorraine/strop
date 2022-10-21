use crate::emulator::Emulator;
use mos6502::cpu::CPU;

#[derive(Debug)]
pub struct Emulator6502 {
    cpu: CPU,
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
