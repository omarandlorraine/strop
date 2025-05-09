use crate::z80::Emulator;
use crate::z80::Insn;
use crate::RunError;

/// Represents a Z80 subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;

impl crate::Run<Emulator> for Subroutine {
    fn run(&self, emulator: &mut Emulator) -> crate::RunResult<()> {
        use crate::Encode;
        use iz80::Machine;

        // Write the subroutine to the beginning of the emulated CPU's address space
        let encoding = self.encode();
        for (addr, val) in encoding.iter().enumerate() {
            // TODO: This will panic if the program grows too long to fit in a Z80's address space
            emulator.machine.poke(addr.try_into().unwrap(), *val);
        }
        emulator.cpu.registers().set_pc(0);

        // Put a value of 0xAAAA at the top of stack (this will be the return address)
        emulator.machine.poke(0x7fff, 0xaa);
        emulator.machine.poke(0x7ffe, 0xaa);
        emulator.cpu.registers().set16(iz80::Reg16::SP, 0x7ffe);

        let end_of_subroutine = encoding.len() as u16;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..1000 {
            let pc = emulator.cpu.registers().pc();
            let sp = emulator.cpu.registers().get16(iz80::Reg16::SP);

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
            emulator.cpu.execute_instruction(&mut emulator.machine);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }
}

#[cfg(test)]
mod iter {
    #[test]
    fn iter() {
        use crate::z80::Emulator;
        use crate::z80::Subroutine;
        use crate::BruteforceSearch;
        use crate::Disassemble;
        use crate::Run;
        use crate::Step;

        let mut subroutine = Subroutine::first();

        assert!(subroutine.run(&mut Emulator::default()).is_ok());
        for _ in 0..5 {
            subroutine.dasm();
            assert!(subroutine.run(&mut Emulator::default()).is_ok());
            subroutine.step();
        }
    }
}

#[cfg(test)]
mod amok {
    #[test]
    fn stack_overflow() {
        use crate::z80::Emulator;
        use crate::z80::Insn;
        use crate::z80::Subroutine;
        use crate::Goto;
        use crate::Run;

        let mut subroutine = Subroutine::default();
        subroutine.goto(&[
            Insn::new(&[0xc5]), // push bc
            Insn::new(&[0xc9]), // ret
        ]);

        assert!(subroutine.run(&mut Emulator::default()).is_err());
    }
}
