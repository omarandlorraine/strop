use crate::m65c02::isa::Insn;
use crate::Fixup;
use crate::RunError;
use crate::Subroutine;
use mos6502::memory::Memory;
use strop::Candidate;


impl<V: std::default::Default+mos6502::Variant> Subroutine<Insn<V>> for Candidate<Insn<V>> {
    type Machine = mos6502::cpu::CPU<Memory, mos6502::instruction::Nmos6502>;

    fn fixup(subroutine: &Self) -> Option<Fixup<Insn<V>>> {
        let end = subroutine.instructions.len() - 1;
        let last_instruction = subroutine.instructions[end];
        
        let rts = Insn::<V>([0x60, 0, 0]);

        if last_instruction != rts {
            return Some(Fixup {
                offset: end,
                replacement: rts,
            });
        }
        None
    }

    fn initial_state() -> Self::Machine {
        Self::Machine::new(Memory::new(), mos6502::instruction::Nmos6502)
    }

    fn run(state: &mut Self::Machine, subroutine: Self) -> Result<(), RunError> {
        use mos6502::memory::Bus;
        let location = 0x0200;
        let return_address = 0x0300;
        let stack_start = 0x80;
        let stack_max = 0x0f;
        let program_length: u16 = subroutine.encode().len().try_into().unwrap();

        state.memory.set_bytes(location, &subroutine.encode());

        state.registers.stack_pointer = mos6502::registers::StackPointer(stack_start);
        state.registers.program_counter = return_address;

        state.execute_instruction((
            mos6502::instruction::Instruction::JSR,
            mos6502::instruction::OpInput::UseAddress(location),
        ));

        for _ in 0..1000 {
            if state.registers.stack_pointer
                < mos6502::registers::StackPointer(stack_start - stack_max)
            {
                return Err(RunError::StackUnderflow);
            }

            if state.registers.stack_pointer > mos6502::registers::StackPointer(stack_start) {
                return Err(RunError::StackOverflow);
            }

            if state.registers.stack_pointer == mos6502::registers::StackPointer(stack_start) {
                if state.registers.program_counter == return_address {
                    return Ok(());
                } else {
                    return Err(RunError::DidntReturn);
                }
            }

            if !(location..location + program_length).contains(&state.registers.program_counter) {
                return Err(RunError::ProgramCounterOutOfBounds);
            }

            state.single_step();
        }

        Err(RunError::DidntReturn)
    }
}
