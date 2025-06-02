use crate::armv4t::Insn;
use crate::RunError;
use crate::Sequence;
use armv4t_emu::{Cpu, ExampleMem};

/// A basic emulator
#[derive(Default)]
pub struct Emulator {
    /// The emulated system's memory
    pub mem: ExampleMem,
    /// The emulated system's CPU
    pub cpu: Cpu,
}

impl Emulator {
    /// Constructs a new emulator, with the function's parameters put in place
    pub fn init<Parameters: crate::armv4t::aapcs32::ParameterList>(
        parameters: &Parameters,
    ) -> Self {
        let mut emu = Self::default();
        parameters.put_list(&mut emu);
        emu
    }

    /// Calls a subroutine
    pub fn call_subroutine(&mut self, subroutine: &Sequence<Insn>) -> crate::RunResult<()> {
        // TODO: check the thing ends in a `bx $lr` instruction
        use crate::Encode;
        use armv4t_emu::reg;
        use armv4t_emu::Memory;
        const RETURN_ADDRESS: u32 = 0x5678;
        const BOTTOM_OF_STACK: u32 = 0x1000;
        let mode = self.cpu.mode();

        // Write the subroutine to the beginning of the emulated CPU's address space
        let encoding = subroutine.encode();
        for (addr, val) in encoding.iter().enumerate() {
            self.mem.w32(addr as u32, *val);
        }
        self.cpu.reg_set(mode, reg::PC, 0);
        self.cpu.reg_set(mode, reg::SP, BOTTOM_OF_STACK);
        self.cpu.reg_set(mode, reg::LR, RETURN_ADDRESS);

        let end_of_subroutine = Encode::<u8>::encode(subroutine).len() as u32;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..10 {
            let pc = self.cpu.reg_get(mode, reg::PC);
            let sp = self.cpu.reg_get(mode, reg::SP);

            if pc == RETURN_ADDRESS && sp == BOTTOM_OF_STACK {
                // Expected values for PC and SP mean that the subroutine has returned
                return Ok(());
            }
            if sp > 0x1000 {
                // Stack underflow; this is not going to go well.
                return Err(RunError::RanAmok);
            }
            if pc > end_of_subroutine {
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            self.cpu.step(&mut self.mem);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }
}

impl std::fmt::Debug for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.cpu)
    }
}
