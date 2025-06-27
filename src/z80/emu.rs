use crate::Sequence;
use crate::z80::Insn;
use crate::{RunError, RunResult};
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
    /// Constructs an instance of the emulator, with the paramers put in place
    pub fn init<Parameters: crate::z80::sdcccall1::ParameterList>(params: &Parameters) -> Self {
        let mut r = Self::default();
        params.put(&mut r);
        r
    }

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

    /// Runs a subroutine in the emulator
    pub fn call_subroutine(&mut self, sequence: &Sequence<Insn>) -> RunResult<()> {
        use crate::Encode;
        use iz80::Machine;

        // Write the subroutine to the beginning of the emulated CPU's address space
        let encoding = sequence.encode();
        for (addr, val) in encoding.iter().enumerate() {
            // TODO: This will panic if the program grows too long to fit in a Z80's address space
            self.machine.poke(addr.try_into().unwrap(), *val);
        }
        self.cpu.registers().set_pc(0);

        // Put a value of 0xAAAA at the top of stack (this will be the return address)
        self.machine.poke(0x7fff, 0xaa);
        self.machine.poke(0x7ffe, 0xaa);
        self.cpu.registers().set16(iz80::Reg16::SP, 0x7ffe);

        let end_of_subroutine = encoding.len() as u16;

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
