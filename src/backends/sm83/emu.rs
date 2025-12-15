use mizu_core::cpu::Cpu;
use mizu_core::cpu::CpuBusProvider;

struct Bus {
    mem: [u8; 65536],
}

impl Default for Bus {
    fn default() -> Self {
        Self { mem: [0; 65536] }
    }
}

impl CpuBusProvider for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }

    fn take_next_interrupt(&mut self) -> Option<mizu_core::memory::interrupts::InterruptType> {
        unreachable!();
    }

    fn peek_next_interrupt(&mut self) -> Option<mizu_core::memory::interrupts::InterruptType> {
        None
    }

    fn is_hdma_running(&mut self) -> bool {
        false
    }
    fn enter_stop_mode(&mut self) {}
    fn stopped(&self) -> bool {
        false
    }
    fn trigger_write_oam_bug(&mut self, _: u16) {}
    fn trigger_read_write_oam_bug(&mut self, _: u16) {}
    fn read_no_oam_bug(&mut self, addr: u16) -> u8 {
        self.read(addr)
    }
}

/// Emulates a SM83 and its 64K of memory
#[derive(Default)]
pub struct Emu {
    cpu: Cpu,
    bus: Bus,
}

impl std::fmt::Debug for Emu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::backends::x80::EmuInterface;
        write!(
            f,
            "strop::sm83::emu::Emu {{ a: {}, pc: {}, sp: {} }}",
            self.get_a(),
            self.get_pc(),
            self.get_sp()
        )
    }
}

impl crate::backends::x80::EmuInterface for Emu {
    fn set_a(&mut self, val: u8) {
        self.cpu.reg_a = val;
    }
    fn set_b(&mut self, val: u8) {
        self.cpu.reg_b = val;
    }
    fn set_c(&mut self, val: u8) {
        self.cpu.reg_c = val;
    }
    fn set_d(&mut self, val: u8) {
        self.cpu.reg_d = val;
    }
    fn set_e(&mut self, val: u8) {
        self.cpu.reg_e = val;
    }
    fn set_h(&mut self, val: u8) {
        self.cpu.reg_h = val;
    }
    fn set_l(&mut self, val: u8) {
        self.cpu.reg_l = val;
    }

    fn get_a(&self) -> u8 {
        self.cpu.reg_a
    }

    fn get_b(&self) -> u8 {
        self.cpu.reg_b
    }

    fn get_d(&self) -> u8 {
        self.cpu.reg_d
    }

    fn get_e(&self) -> u8 {
        self.cpu.reg_e
    }

    fn get_c(&self) -> u8 {
        self.cpu.reg_c
    }

    fn get_hl(&self) -> u16 {
        u16::from_be_bytes([self.cpu.reg_h, self.cpu.reg_l])
    }

    fn get_pc(&self) -> u16 {
        self.cpu.reg_pc
    }

    fn get_sp(&self) -> u16 {
        self.cpu.reg_sp
    }

    fn push(&mut self, val: u16) {
        self.cpu.stack_push(val, &mut self.bus)
    }

    fn pop(&mut self) -> u16 {
        self.cpu.stack_pop(&mut self.bus)
    }

    fn call(&mut self, subroutine: Vec<u8>) -> crate::RunResult<()> {
        use crate::RunError;
        // Write the subroutine to the beginning of the emulated CPU's address space
        for (addr, val) in subroutine.iter().enumerate() {
            // TODO: This will panic if the program grows too long to fit in a Z80's address space
            self.bus.write(addr.try_into().unwrap(), *val);
        }

        // Put a value of 0xAAAA at the top of stack (this will be the return address)
        self.cpu.reg_sp = 0x8000;
        self.push(0xaaaa);

        let end_of_subroutine = subroutine.len() as u16;

        // Single step through the subroutine until it returns or runs amok
        for _ in 0..1000 {
            let pc = self.get_pc();
            let sp = self.get_sp();

            if pc == 0xaaaa && sp == 0x0000 {
                // Expected values for PC and SP mean that the subroutine has returned
                return Ok(());
            }
            if pc > end_of_subroutine {
                // the program counter is out of bounds; the subroutine seems to have run amok
                return Err(RunError::RanAmok);
            }
            self.cpu.next_instruction(&mut self.bus);
        }
        // Never even returned!
        Err(RunError::RanAmok)
    }

    fn poke(&mut self, addr: u16, val: u8) {
        self.bus.write(addr, val);
    }

    fn peek(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn single_step(&mut self) -> crate::RunResult<()> {
        self.cpu.next_instruction(&mut self.bus);
        Ok(())
    }
}
