//! Module containing emulators for the 6502, with the necessary additions making them suitable for
//! use with strop.
use crate::Candidate;
use crate::Emulator;
use crate::Instruction;
use robo6502;
use std::convert::TryInto;

/// Emulates 64 kilobytes of RAM
#[derive(Debug)]
pub struct Ram64K {
    mem: [u8; 65536],
}

impl Default for Ram64K {
    fn default() -> Self {
        Self { mem: [0; 65536] }
    }
}

impl robo6502::Sys for Ram64K {
    fn read(&mut self, addr: u16) -> Option<u8> {
        Some(self.mem[addr as usize])
    }

    fn write(&mut self, addr: u16, data: u8) -> Option<()> {
        self.mem[addr as usize] = data;
        Some(())
    }
}

/// This emulates the original 6502, as found in the Commodore 64. Illegal instructions are
/// supported as well.
#[derive(Default, Debug)]
pub struct Nmos6502 {
    /// The CPU's internal state
    pub cpu: robo6502::Nmos,
    /// The memory
    pub mem: Ram64K,
}

trait Interface6502 {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, val: u8);
}

impl Interface6502 for Nmos6502 {
    fn read(&self, addr: u16) -> Option<u8> {
        Some(self.mem.mem[addr as usize])
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem.mem[addr as usize] = val;
    }
}

impl Interface6502 for Cmos6502 {
    fn read(&self, addr: u16) -> Option<u8> {
        Some(self.mem.mem[addr as usize])
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem.mem[addr as usize] = val;
    }
}

impl<T: Instruction> Emulator<T> for Nmos6502 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        use robo6502::Cpu;
        use robo6502::Sys;
        let encoding = candidate.encode();
        let org: u16 = addr.try_into().unwrap();
        let end: u16 = (addr + encoding.len()).try_into().unwrap();

        let mut pointer = org;
        for byte in encoding {
            self.mem.write(pointer, byte);
            pointer += 1;
        }
        self.cpu.set_pc(org);

        for _ in 0..1000 {
            self.cpu.run_instruction(&mut self.mem);
            let pc = self.cpu.pc();
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}

impl Nmos6502 {
    /// return value of accumulator
    pub fn a(&self) -> u8 {
        use robo6502::Cpu;
        self.cpu.a()
    }

    #[cfg(test)]
    pub fn run_one_instruction(
        &mut self,
        insn: crate::mos6502::instruction_set::Nmos6502Instruction,
    ) -> usize {
        use robo6502::Cpu;
        use robo6502::Sys;
        let encoding = insn.encode();
        let org: u16 = 0x0200;

        let mut pointer = org;
        for byte in encoding {
            self.mem.write(pointer, byte);
            pointer += 1;
        }
        self.cpu.set_pc(org);

        self.cpu
            .run_instruction(&mut self.mem)
            .unwrap_or_else(|| panic!("Could not execute instruction {}", insn));

        self.cpu.pc().wrapping_sub(org) as usize
    }
}

/// This emulates the original 6502, as found in the Commodore 64. Illegal instructions are
/// supported as well.
#[derive(Default, Debug)]
pub struct Cmos6502 {
    /// The CPU's internal state
    pub cpu: robo6502::Cmos,
    /// The memory
    pub mem: Ram64K,
}

impl<T: Instruction> Emulator<T> for Cmos6502 {
    fn run(&mut self, addr: usize, candidate: &Candidate<T>) {
        use robo6502::Cpu;
        use robo6502::Sys;
        let encoding = candidate.encode();
        let org: u16 = addr.try_into().unwrap();
        let end: u16 = (addr + encoding.len()).try_into().unwrap();

        let mut pointer = org;
        for byte in encoding {
            self.mem.write(pointer, byte);
            pointer += 1;
        }
        self.cpu.set_pc(org);

        for _ in 0..1000 {
            self.cpu.run_instruction(&mut self.mem);
            let pc = self.cpu.pc();
            if pc < org {
                break;
            }
            if pc > end {
                break;
            }
        }
    }
}

impl Cmos6502 {
    /// return value of accumulator
    pub fn a(&self) -> u8 {
        use robo6502::Cpu;
        self.cpu.a()
    }

    #[cfg(test)]
    pub fn run_one_instruction(
        &mut self,
        insn: crate::mos6502::instruction_set::Cmos6502Instruction,
    ) -> usize {
        use robo6502::Cpu;
        use robo6502::Sys;
        let encoding = insn.encode();
        let org: u16 = 0x0200;

        let mut pointer = org;
        for byte in encoding {
            self.mem.write(pointer, byte);
            pointer += 1;
        }
        self.cpu.set_pc(org);

        self.cpu
            .run_instruction(&mut self.mem)
            .unwrap_or_else(|| panic!("Could not execute instruction {}", insn));

        self.cpu.pc().wrapping_sub(org) as usize
    }
}
