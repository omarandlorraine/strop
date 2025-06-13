use crate::m68k::Insn;
use crate::Encode;
use crate::RunError;
use crate::RunResult;
use crate::Sequence;

use m68000::*;

const MEM_SIZE: u32 = 65536;
#[derive(Debug)]
pub struct Memory([u8; MEM_SIZE as usize]); // Define your memory management system.

impl Memory {
    pub fn new() -> Self {
        Self([0; MEM_SIZE as usize])
    }
}

impl MemoryAccess for Memory {
    // Implement the MemoryAccess trait.
    fn get_byte(&mut self, addr: u32) -> Option<u8> {
        if addr < MEM_SIZE {
            Some(self.0[addr as usize])
        } else {
            None
        }
    }

    fn get_word(&mut self, addr: u32) -> Option<u16> {
        Some((self.get_byte(addr)? as u16) << 8 | self.get_byte(addr + 1)? as u16)
    }

    fn set_byte(&mut self, addr: u32, value: u8) -> Option<()> {
        if addr < MEM_SIZE {
            self.0[addr as usize] = value;
            Some(())
        } else {
            None
        }
    }

    fn set_word(&mut self, addr: u32, value: u16) -> Option<()> {
        self.set_byte(addr, (value >> 8) as u8)?;
        self.set_byte(addr + 1, value as u8)
    }

    fn reset_instruction(&mut self) {}
}

/// The 68000 emulator.
#[derive(Debug)]
pub struct Emulator {
    /// The machine
    pub memory: Memory,
    /// The CPU
    pub cpu: M68000<m68000::cpu_details::Mc68000>,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            cpu: M68000::<m68000::cpu_details::Mc68000>::new(),
        }
    }

    pub fn set_d0(&mut self, value: u32) {
        self.cpu.regs.d[0] = std::num::Wrapping(value);
    }

    pub fn get_d0(&self) -> u32 {
        self.cpu.regs.d[0].0
    }

    pub fn call_subroutine(&mut self, seq: &Sequence<Insn>) -> RunResult<()> {
        let bin = Encode::<u16>::encode(seq);

        const START_ADDRESS: u32 = 418;
        let top_of_stack = self.cpu.regs.sp();

        for (address, word) in bin.iter().enumerate() {
            self.memory
                .set_word(START_ADDRESS + ((address * 2) as u32), *word)
                .unwrap();
        }

        let address_of_last_instruction =
            std::num::Wrapping(START_ADDRESS + (((bin.len() - 1) * 2) as u32));
        let address_of_first_instruction = std::num::Wrapping(START_ADDRESS);

        self.cpu.regs.pc = std::num::Wrapping(START_ADDRESS);

        for _ in 0..40000 {
            if self.cpu.regs.pc == address_of_last_instruction {
                // The subroutine has reached the last instruction (which is assumed to be a return
                // instruction)
                if self.cpu.regs.sp() != top_of_stack {
                    // the stack either underflowed at the last minute or things have been left on
                    // there
                    return Err(RunError::RanAmok);
                }
                return Ok(());
            }
            self.cpu.interpreter(&mut self.memory);
            if !(address_of_first_instruction..address_of_last_instruction)
                .contains(&self.cpu.regs.pc)
            {
                // The subroutine has jumped to outside of itself.
                return Err(RunError::RanAmok);
            }
            if self.cpu.regs.sp() > top_of_stack {
                // The subroutine has underflowed the stack
                return Err(RunError::RanAmok);
            }
        }

        // Perhaps the subroutine contained an infinite loop or otherwise took too long.
        return Err(RunError::RanAmok);
    }
}
