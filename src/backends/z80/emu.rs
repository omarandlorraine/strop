use crate::backends::x80::EmuInterface;
use crate::{RunError, RunResult};
use iz80::{Cpu, PlainMachine, Reg8, Reg16};

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

impl EmuInterface for Emulator {
    fn set_a(&mut self, val: u8) {
        self.cpu.registers().set_a(val);
    }
    fn set_b(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::B, val);
    }
    fn set_c(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::C, val);
    }
    fn set_d(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::D, val);
    }
    fn set_e(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::E, val);
    }
    fn set_h(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::H, val);
    }
    fn set_l(&mut self, val: u8) {
        self.cpu.registers().set8(Reg8::L, val);
    }

    fn get_a(&self) -> u8 {
        self.cpu.immutable_registers().a()
    }
    fn get_b(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::B)
    }
    fn get_c(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::C)
    }
    fn get_d(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::D)
    }
    fn get_e(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::E)
    }
    fn get_h(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::H)
    }
    fn get_l(&self) -> u8 {
        self.cpu.immutable_registers().get8(Reg8::L)
    }

    fn get_pc(&self) -> u16 {
        self.cpu.immutable_registers().pc()
    }
    fn get_sp(&self) -> u16 {
        self.cpu.immutable_registers().get16(Reg16::SP)
    }

    fn pop(&mut self) -> u16 {
        use iz80::Machine;
        let sp = self.cpu.registers().get16(iz80::Reg16::SP);
        let r = self.machine.peek16(sp);
        self.cpu.registers().set16(iz80::Reg16::SP, sp + 2);
        r
    }

    fn push(&mut self, val: u16) {
        use iz80::Machine;
        let sp = self.cpu.registers().get16(iz80::Reg16::SP);
        self.machine.poke16(sp - 2, val);
        self.cpu.registers().set16(iz80::Reg16::SP, sp - 2);
    }

    fn get_hl(&self) -> u16 {
        self.cpu.immutable_registers().get16(Reg16::HL)
    }

    /// Runs a subroutine in the emulator
    fn call(&mut self, sequence: Vec<u8>) -> RunResult<()> {
        use iz80::Machine;

        // Write the subroutine to the beginning of the emulated CPU's address space
        for (addr, val) in sequence.iter().enumerate() {
            // TODO: This will panic if the program grows too long to fit in a Z80's address space
            self.machine.poke(addr.try_into().unwrap(), *val);
        }
        self.cpu.registers().set_pc(0);

        // Put a value of 0xAAAA at the top of stack (this will be the return address)
        self.machine.poke(0x7fff, 0xaa);
        self.machine.poke(0x7ffe, 0xaa);
        self.cpu.registers().set16(iz80::Reg16::SP, 0x7ffe);

        let end_of_subroutine = sequence.len() as u16;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..1000 {
            let pc = self.cpu.registers().pc();
            let sp = self.cpu.registers().get16(iz80::Reg16::SP);

            if pc == 0xaaaa && sp == 0x8000 {
                // Expected values for PC and SP mean that the subroutine has returned
                return Ok(());
            }
            if sp > 0x7ffe {
                // Stack underflow; this is not going to go well.
                return Err(RunError::RanAmok);
            }
            if pc > end_of_subroutine {
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            self.cpu.execute_instruction(&mut self.machine);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }
}
