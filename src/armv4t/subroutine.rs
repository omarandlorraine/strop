use crate::armv4t::Emulator;
use crate::armv4t::Insn;
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
        use armv4t_emu::Memory;
        use armv4t_emu::reg;
        const return_address: u32 = 0x5678;
        const bottom_of_stack: u32 = 0x1000;
        let mode = emulator.cpu.mode();
        dbg!(mode);

        // Write the subroutine to the beginning of the emulated CPU's address space
        let encoding = self.encode();
        for (addr, val) in encoding.iter().enumerate() {
            dbg!((addr as u32, *val));
            emulator.mem.w32(addr as u32, *val);
        }
        emulator.cpu.reg_set(mode, reg::PC, 0);
        emulator.cpu.reg_set(mode, reg::SP, bottom_of_stack);
        emulator.cpu.reg_set(mode, reg::LR, return_address);

        let end_of_subroutine = Encode::<u8>::encode(self).len() as u32;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..10 {
            let pc = emulator.cpu.reg_get(mode, reg::PC);
            let sp = emulator.cpu.reg_get(mode, reg::SP);
            let lr = emulator.cpu.reg_get(mode, reg::SP);

            dbg!(pc);
            dbg!(sp);
            dbg!(lr);
            if pc == return_address && sp == bottom_of_stack {
                // Expected values for PC and SP mean that the subroutine has returned
                return Ok(());
            }
            if sp > 0x1000 {
                // Stack underflow; this is not going to go well.
                dbg!("stack underflow");
                return Err(RunError::RanAmok);
            }
            if pc > end_of_subroutine {
                dbg!("PC out of bounds");
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            emulator.cpu.step(&mut emulator.mem);
        }
        // Never even returned!
        dbg!("never even returned!");
        Err(RunError::RanAmok)
    }
}

#[cfg(test)]
mod okay {
    #[test]
    fn all_returning_instruction() {
        use crate::BruteforceSearch;
        use crate::armv4t::Subroutine;
        use crate::armv4t::Emulator;
        use crate::Disassemble;
        use crate::Encode;
        use crate::Run;

        let mut subroutine = Subroutine::default();
        let mut emu = Emulator::default();
        subroutine.step();

        while Encode::<u32>::encode(&subroutine).len() == 1 {
            println!("attempt:");
            subroutine.dasm();

            assert!(subroutine.run(&mut emu).is_ok());
            println!("returned");
            subroutine.step();
        }
    }
}
