use crate::armv4t::Emulator;
use crate::armv4t::Insn;
use crate::RunError;

/// Represents an ARMv4T subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;

impl crate::Run<Emulator> for Subroutine {
    fn run(&self, emulator: &mut Emulator) -> crate::RunResult<()> {
        use crate::Encode;
        use armv4t_emu::reg;
        use armv4t_emu::Memory;
        const RETURN_ADDRESS: u32 = 0x5678;
        const BOTTOM_OF_STACK: u32 = 0x1000;
        let mode = emulator.cpu.mode();

        // Write the subroutine to the beginning of the emulated CPU's address space
        let encoding = self.encode();
        for (addr, val) in encoding.iter().enumerate() {
            emulator.mem.w32(addr as u32, *val);
        }
        emulator.cpu.reg_set(mode, reg::PC, 0);
        emulator.cpu.reg_set(mode, reg::SP, BOTTOM_OF_STACK);
        emulator.cpu.reg_set(mode, reg::LR, RETURN_ADDRESS);

        let end_of_subroutine = Encode::<u8>::encode(self).len() as u32;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..10 {
            let pc = emulator.cpu.reg_get(mode, reg::PC);
            let sp = emulator.cpu.reg_get(mode, reg::SP);

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
            emulator.cpu.step(&mut emulator.mem);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }
}

#[cfg(test)]
mod okay {
    #[test]
    fn all_returning_instructions() {
        use crate::armv4t::Emulator;
        use crate::armv4t::Subroutine;
        use crate::BruteforceSearch;
        use crate::Disassemble;
        use crate::Run;
        use crate::Step;

        let mut subroutine = Subroutine::first();
        let mut emu = Emulator::default();

        println!("attempt:{:?}", subroutine.analyze());
        subroutine.dasm();

        assert!(subroutine.run(&mut emu).is_ok());
        println!("returned");

        subroutine.step();
        println!("attempt:{:?}", subroutine.analyze());
        println!("attempt:{:?}", subroutine.analyze());
    }
}
