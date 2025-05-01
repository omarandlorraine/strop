use mos6502::cpu::CPU;
use mos6502::memory::Memory;

/// A simple 6502 emulator
#[derive(Debug)]
pub struct Emulator<Variant: mos6502::Variant> {
    /// The emulator's CPU
    pub cpu: CPU<Memory, Variant>,
}

impl<Variant: mos6502::Variant + Default> Default for Emulator<Variant> {
    fn default() -> Self {
        let mut cpu = CPU::new(Memory::new(), Variant::default());
        cpu.registers.stack_pointer = mos6502::registers::StackPointer(0xff);
        Self { cpu }
    }
}
