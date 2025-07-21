use crate::Sequence;
use crate::m6502::Insn;
use mos6502::cpu::CPU;
use mos6502::memory::Memory;

/// A simple 6502 emulator
#[derive(Debug)]
pub struct Emulator<Variant: mos6502::Variant> {
    /// The emulator's CPU
    pub cpu: CPU<Memory, Variant>,
}

impl<Variant: mos6502::Variant + Default> Default for Emulator<Variant> {
    fn default() -> Self {
        let mut cpu = CPU::new(Memory::new(), Variant::default());
        cpu.registers.stack_pointer = mos6502::registers::StackPointer(0xff);
        Self { cpu }
    }
}

impl<Variant: mos6502::Variant> Emulator<Variant> {
    /// Run the subroutine on a given emulator
    pub fn run(&mut self, subroutine: &Sequence<Insn<Variant>>) -> crate::RunResult<()> {
        use crate::Encode;
        use mos6502::memory::Bus;

        const RETURN_ADDRESS: u16 = 0x5678;
        const SUBROUTINE_ADDRESS: u16 = 0x2345;

        let old_stack_pointer = self.cpu.registers.stack_pointer;

        println!("about to run jsr");
        self.cpu.registers.program_counter = RETURN_ADDRESS;
        dbg!(
            self.cpu.registers.program_counter,
            self.cpu.registers.stack_pointer.0
        );
        self.cpu.execute_instruction((
            mos6502::instruction::Instruction::JSR,
            mos6502::instruction::OpInput::UseAddress(SUBROUTINE_ADDRESS),
        ));

        self.cpu
            .memory
            .set_bytes(SUBROUTINE_ADDRESS, &subroutine.encode());
        dbg!(
            self.cpu.registers.program_counter,
            self.cpu.registers.stack_pointer.0
        );

        for _ in 0..1000 {
            println!("about to single step");
            dbg!(
                self.cpu.registers.program_counter,
                self.cpu.registers.stack_pointer.0
            );
            self.cpu.single_step();

            if old_stack_pointer == self.cpu.registers.stack_pointer {
                if self.cpu.registers.program_counter == RETURN_ADDRESS {
                    // the stack pointer and program counter seem to match what we'd expect for a
                    // subroutine that's returned
                    return Ok(());
                } else {
                    // the stack underflowed or something
                    return Err(crate::RunError::RanAmok);
                }
            }

            if self.cpu.registers.program_counter
                > SUBROUTINE_ADDRESS + subroutine.encode().len() as u16
            {
                // the subroutine branched to outside of itself
                return Err(crate::RunError::RanAmok);
            }

            if self.cpu.registers.program_counter < SUBROUTINE_ADDRESS {
                // the subroutine branched to outside of itself
                return Err(crate::RunError::RanAmok);
            }
        }
        Err(crate::RunError::RanAmok)
    }
}
