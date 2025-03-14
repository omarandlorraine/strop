use iz80::*;

/// The Z80 emulator.
#[derive(Default)]
pub struct Emulator {
    /// The machine
    pub machine: PlainMachine,
    /// The CPU
    pub cpu: Cpu,
}

impl std::fmt::Debug for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Emulator {{ a: {}, hl: {} }}",
            self.get_a(),
            self.get_hl()
        )
    }
}

impl Emulator {
    /// Sets the emulator's accumulator to some value
    pub fn set_a(&mut self, val: u8) {
        self.cpu.registers().set_a(val);
    }
    /// Sets the emulator's B register to some value
    pub fn set_b(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::B, val);
    }

    /// Sets the emulator's BC register to some value
    pub fn set_bc(&mut self, val: u16) {
        self.cpu.registers().set16(Reg16::BC, val);
    }

    /// Sets the emulator's HL register to some value
    pub fn set_hl(&mut self, val: u16) {
        self.cpu.registers().set16(Reg16::HL, val);
    }

    /// Returns the value of the accumulator
    pub fn get_a(&self) -> u8 {
        self.cpu.immutable_registers().a()
    }

    /// Returns the value of the emulator's HL register
    pub fn get_hl(&self) -> u16 {
        self.cpu.immutable_registers().get16(Reg16::HL)
    }
}
