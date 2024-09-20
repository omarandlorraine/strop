use crate::z80::Insn;
use crate::Encode;
use crate::Sequence;
use crate::StropError;
use iz80::*;

/// The Z80 emulator.
#[derive(Default)]
pub struct Emulator {
    machine: PlainMachine,
    cpu: Cpu,
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

    /// Puts the sequence of instructions into the emulator's memory, starting at address 0, and
    /// single steps until the end of the program is reached.
    pub fn run(&mut self, program: &Sequence<Insn>) -> Result<(), StropError> {
        let encoding = program.encode();

        for (addr, val) in encoding.iter().enumerate() {
            // TODO: This will panic if the program grows too long to fit in a Z80's address space
            self.machine.poke(addr.try_into().unwrap(), *val);
        }
        self.cpu.registers().set_pc(0);

        for _ in 0..1000 {
            if usize::from(self.cpu.registers().pc()) > encoding.len() {
                return Ok(());
            }
            self.cpu.execute_instruction(&mut self.machine);
        }
        Err(StropError::DidntReturn)
    }
}
