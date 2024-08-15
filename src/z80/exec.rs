use crate::z80::isa::Insn;
use crate::Fixup;
use crate::RunError;
use crate::Subroutine;
use iz80::*;
use strop::Candidate;

const RET: Insn = Insn([0xc9, 0, 0, 0, 0]);

pub struct Z80State {
    mach: PlainMachine,
    cpu: Cpu,
}

impl Subroutine<Insn> for Candidate<Insn> {
    type Machine = Z80State;

    fn fixup(subroutine: &Self) -> Option<Fixup<Insn>> {
        let end = subroutine.instructions.len() - 1;
        let last_instruction = subroutine.instructions[end];

        if last_instruction != RET {
            return Some(Fixup {
                offset: end,
                replacement: RET,
            });
        }
        None
    }

    fn initial_state() -> Self::Machine {
        Z80State {
            mach: PlainMachine::new(),
            cpu: Cpu::new(),
        }
    }

    fn run(state: &mut Self::Machine, subroutine: Self) -> Result<(), RunError> {
        use strop::Instruction;

        let location = 0x0200;
        let call_location = 0x0300;
        let return_address = 0x0303;
        let stack_start = 0x0800;
        let stack_boundary = stack_start - 0x0f;
        let program_length: u16 = subroutine.encode().len().try_into().unwrap();

        let code = subroutine.encode();
        for (offset, byte) in code.iter().enumerate() {
            state.mach.poke(location + offset as u16, *byte);
        }

        let call = Insn([
            0xcd,
            location.to_le_bytes()[0],
            location.to_le_bytes()[1],
            0,
            0,
        ]);
        for (offset, byte) in call.encode().iter().enumerate() {
            state.mach.poke(call_location + offset as u16, *byte);
        }

        state.cpu.registers().set_pc(call_location);
        state.cpu.execute_instruction(&mut state.mach);

        assert_eq!(state.cpu.registers().pc(), location);

        for _ in 0..1000 {
            let pc = state.cpu.registers().pc();
            let sp = state.cpu.registers().get16(Reg16::SP);

            if pc == return_address {
                if sp == stack_start {
                    return Ok(());
                } else {
                    return Err(RunError::DidntReturn);
                }
            }

            if sp < stack_boundary {
                return Err(RunError::StackOverflow);
            }

            if sp > stack_start {
                return Err(RunError::StackUnderflow);
            }

            if !(location..location + program_length).contains(&state.cpu.registers().pc()) {
                return Err(RunError::ProgramCounterOutOfBounds);
            }
            state.cpu.execute_instruction(&mut state.mach);
        }

        Err(RunError::DidntReturn)
    }
}
