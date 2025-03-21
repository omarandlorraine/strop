use m68000::*;

const MEM_SIZE: u32 = 65536;
#[derive(Debug)]
pub struct Memory([u8; MEM_SIZE as usize]); // Define your memory management system.

impl MemoryAccess for Memory { // Implement the MemoryAccess trait.
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
}
