use crate::m68000::Emulator;
use crate::m68000::Insn;
use crate::RunError;

/// Represents a Z80 subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;

impl Default for Subroutine {
    fn default() -> Self {
        use crate::subroutine::ToSubroutine;
        use crate::Step;
        crate::Sequence::<Insn>::first().to_subroutine()
    }
}

impl crate::Run<Emulator> for Subroutine {
    fn run(&self, emulator: &mut Emulator) -> crate::RunResult<()> {
        use crate::Encode;
        use m68000::MemoryAccess;

        const SUBROUTINE_START: u32 = 0x0400;
        const INITIAL_STACK_POINTER: u32 = 0x0fff;

        // Write the subroutine to the specified location
        let encoding = self.encode();
        for (addr, val) in encoding.iter().enumerate() {
            let addr = addr as u32;
            emulator.memory.set_word(SUBROUTINE_START + addr, *val).unwrap();
        }

        // Write the subroutine's address to the RESET vector and initialize the stack pointer
        // thing as well
        emulator.memory.set_word(0, INITIAL_STACK_POINTER as u16).unwrap();
        emulator.memory.set_word(2, 0).unwrap();
        emulator.memory.set_word(4, SUBROUTINE_START as u16).unwrap();
        emulator.memory.set_word(6, 0).unwrap();

        let end_of_subroutine = std::num::Wrapping(SUBROUTINE_START + encoding.len() as u32);
        let stack_underflow = std::num::Wrapping(INITIAL_STACK_POINTER - 2);

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..1000 {
            let pc = emulator.cpu.regs.pc;
            let sp = emulator.cpu.regs.ssp;

            if pc == end_of_subroutine && sp == std::num::Wrapping(INITIAL_STACK_POINTER) {
                // Expected values for PC and SP mean that the subroutine is about to return
                return Ok(());
            }
            if sp > stack_underflow {
                // Stack underflow; this is not going to go well.
                return Err(RunError::RanAmok);
            }
            if pc > end_of_subroutine {
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            emulator.cpu.cycle(&mut emulator.memory, 1);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }
}
