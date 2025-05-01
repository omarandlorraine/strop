use crate::m6502::Emulator;
use crate::m6502::Insn;

/// Wraps up a `Sequence<Insn>`, that is, a sequence of Z80 instructions, and associates it with
/// static analysis that makes sure it's a valid Z80 subroutine.
pub type Subroutine<V> = crate::Subroutine<Insn<V>, crate::Sequence<Insn<V>>>;

impl<V: mos6502::Variant> Subroutine<V> {
    /// Run the subroutine on a given emulator
    pub fn run(&self, emulator: &mut Emulator<V>) -> crate::RunResult<()> {
        use crate::Encode;
        use mos6502::memory::Bus;

        const RETURN_ADDRESS: u16 = 0x5678;
        const SUBROUTINE_ADDRESS: u16 = 0x2345;

        let old_stack_pointer = emulator.cpu.registers.stack_pointer;

        println!("about to run jsr");
        emulator.cpu.registers.program_counter = RETURN_ADDRESS;
        dbg!(
            emulator.cpu.registers.program_counter,
            emulator.cpu.registers.stack_pointer.0
        );
        emulator.cpu.execute_instruction((
            mos6502::instruction::Instruction::JSR,
            mos6502::instruction::OpInput::UseAddress(SUBROUTINE_ADDRESS),
        ));

        emulator
            .cpu
            .memory
            .set_bytes(SUBROUTINE_ADDRESS, &self.encode());
        dbg!(
            emulator.cpu.registers.program_counter,
            emulator.cpu.registers.stack_pointer.0
        );

        for _ in 0..1000 {
            println!("about to single step");
            dbg!(
                emulator.cpu.registers.program_counter,
                emulator.cpu.registers.stack_pointer.0
            );
            emulator.cpu.single_step();

            if old_stack_pointer == emulator.cpu.registers.stack_pointer {
                if emulator.cpu.registers.program_counter == RETURN_ADDRESS {
                    // the stack pointer and program counter seem to match what we'd expect for a
                    // subroutine that's returned
                    return Ok(());
                } else {
                    // the stack underflowed or something
                    return Err(crate::RunError::RanAmok);
                }
            }

            if emulator.cpu.registers.program_counter
                > SUBROUTINE_ADDRESS + self.encode().len() as u16
            {
                // the subroutine branched to outside of itself
                return Err(crate::RunError::RanAmok);
            }

            if emulator.cpu.registers.program_counter < SUBROUTINE_ADDRESS {
                // the subroutine branched to outside of itself
                return Err(crate::RunError::RanAmok);
            }
        }
        Err(crate::RunError::RanAmok)
    }
}

#[cfg(test)]
mod ok {
    #[test]
    fn ok() {
        use super::Subroutine;
        use crate::m6502::Emulator;
        use crate::m6502::Nmos6502;
        use crate::BruteforceSearch;
        use crate::Disassemble;

        let mut subroutine: Subroutine<Nmos6502> = Subroutine::new(Default::default());
        subroutine.step();
        let mut emu = Emulator::default();
        subroutine.dasm();
        assert!(subroutine.run(&mut emu).is_ok());
    }
}
