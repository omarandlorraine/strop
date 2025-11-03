//! Module implementing O32, a Callable and Searchable complying with MIPS's O32 calling
//! convention.

pub(crate) struct Bus {
    pub kseg1: [u8; 0x10000],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            kseg1: [0; 0x10000],
        }
    }
}

impl trapezoid_core::cpu::BusLine for Bus {
    // just maps the kseg1 memory to the CPU
    fn read_u32(&mut self, addr: u32) -> Result<u32, String> {
        match addr {
            0xBFC00000..=0xBFC0FFFF => {
                let offset = addr - 0xBFC00000;
                let word = u32::from_le_bytes([
                    self.kseg1[offset as usize],
                    self.kseg1[(offset + 1) as usize],
                    self.kseg1[(offset + 2) as usize],
                    self.kseg1[(offset + 3) as usize],
                ]);
                Ok(word)
            }
            _ => Ok(0),
        }
    }

    // just maps the kseg1 memory to the CPU
    fn read_u8(&mut self, addr: u32) -> Result<u8, String> {
        match addr {
            0xBFC00000..=0xBFC0FFFF => {
                let offset = addr - 0xBFC00000;
                Ok(self.kseg1[offset as usize])
            }
            _ => Ok(0),
        }
    }

    // Ability to write to kseg1
    fn write_u32(&mut self, addr: u32, data: u32) -> Result<(), String> {
        match addr {
            0xBFC00000..=0xBFC0FFFF => {
                let offset = addr - 0xBFC00000;
                self.kseg1[offset as usize] = data as u8;
                self.kseg1[(offset + 1) as usize] = (data >> 8) as u8;
                self.kseg1[(offset + 2) as usize] = (data >> 16) as u8;
                self.kseg1[(offset + 3) as usize] = (data >> 24) as u8;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    // Ability to write to kseg1
    fn write_u8(&mut self, addr: u32, data: u8) -> Result<(), String> {
        match addr {
            0xbfc00000..=0xbfc0ffff => {
                let offset = addr - 0xbfc00000;
                self.kseg1[offset as usize] = data;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// Not used; we don't need interrupts in this contexts
impl trapezoid_core::cpu::CpuBusProvider for Bus {
    fn pending_interrupts(&self) -> bool {
        false
    }

    fn should_run_dma(&self) -> bool {
        false
    }
}
