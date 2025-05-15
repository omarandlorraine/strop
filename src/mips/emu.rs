//! A module making possible the execution of MIPS subroutine in emulation
use crate::mips::Insn;
use crate::Encode;
use crate::Sequence;
use trapezoid_core::cpu::{BusLine, Cpu, CpuBusProvider, RegisterType};

struct Bus {
    kseg1: [u8; 0x10000],
    is_done: bool,
}

impl BusLine for Bus {
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

    // just maps the kseg1 memory to the CPU, needed to read the string data
    fn read_u8(&mut self, addr: u32) -> Result<u8, String> {
        match addr {
            0xBFC00000..=0xBFC0FFFF => {
                let offset = addr - 0xBFC00000;
                Ok(self.kseg1[offset as usize])
            }
            _ => Ok(0),
        }
    }

    // Ability to write to the stack location, here we allow whole kseg1, but can be limited as
    // needed
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

    // support our custom registers
    fn write_u8(&mut self, addr: u32, data: u8) -> Result<(), String> {
        match addr {
            // exit
            0x0 => {
                println!("Write to address 0x0: {:08X}, exiting", data);
                self.is_done = data != 0x0;
                Ok(())
            }
            // write a character
            0x4 => {
                print!("{}", data as char);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

// Not used for now, we don't have interrupts handling from the hardware
// If we do, when `pending_interrupts` returns true, the CPU will jump to the interrupt vector with
// cause `Interrupt` (not available for public API), but the interrupt vector is
// `0xBFC00180` or `0x80000080`.
impl CpuBusProvider for Bus {
    fn pending_interrupts(&self) -> bool {
        false
    }

    fn should_run_dma(&self) -> bool {
        false
    }
}

/// Trait for types which may be used as a function's parameter list.
pub trait Parameters {
    /// Puts the parameters into the emulator in the expected way.
    fn install(self, cpu: &mut Cpu);
    /// Performs dataflow analysis; ensuring that the sequence reads from the necessary argument
    /// registers.
    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>>;
}

/// Trait for types which may be used as a function's return value
pub trait ReturnValue {
    /// Gets the parameters out of the emulator's register file
    fn extract(cpu: &Cpu) -> Self;
    /// Performs dataflow analysis; ensuring that the sequence writes to V0 (and, V1 if necessary)
    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>>;
}

impl Parameters for u8 {
    fn install(self, cpu: &mut Cpu) {
        cpu.registers_mut().write(RegisterType::A0, self as u32)
    }

    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>> {
        crate::dataflow::expect_read(seq, &RegisterType::A0)
    }
}

impl ReturnValue for u8 {
    fn extract(cpu: &Cpu) -> Self {
        cpu.registers().read(RegisterType::V0) as u8
    }

    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>> {
        crate::dataflow::expect_write(seq, &RegisterType::V0)
    }
}

impl Parameters for f32 {
    fn install(self, cpu: &mut Cpu) {
        cpu.registers_mut().write(RegisterType::A0, self.to_bits())
    }

    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>> {
        crate::dataflow::expect_read(seq, &RegisterType::A0)?;
        crate::dataflow::uninitialized(seq, &RegisterType::A1)?;
        crate::dataflow::uninitialized(seq, &RegisterType::A2)?;
        crate::dataflow::uninitialized(seq, &RegisterType::A3)
    }
}

impl ReturnValue for f32 {
    fn extract(cpu: &Cpu) -> Self {
        Self::from_bits(cpu.registers().read(RegisterType::V0))
    }

    fn analyze_this(seq: &Sequence<Insn>) -> Result<(), crate::StaticAnalysis<Insn>> {
        crate::dataflow::expect_write(seq, &RegisterType::V0)
    }
}

/// Puts the arguments into the CPU's registers, then puts the subroutine into kseg1, and then
/// calls the subroutine. After this, it returns the return value.
pub fn call<P: Parameters, R: ReturnValue>(
    subroutine: &crate::mips::Subroutine,
    params: P,
) -> crate::RunResult<R> {
    let mut cpu = Cpu::new();
    let mut kseg1 = [0; 0x10000];
    let subroutine = subroutine.encode();
    kseg1[0..subroutine.len()].copy_from_slice(&subroutine);

    let mut bus = Bus {
        kseg1,
        is_done: false,
    };

    let end_pc = 0xBFC00000 + subroutine.len() as u32;

    params.install(&mut cpu);

    for _ in 0..10000 {
        if cpu.registers().read(RegisterType::Pc) == end_pc {
            return Ok(R::extract(&cpu));
        }
        cpu.clock(&mut bus, 1);
    }
    Err(crate::RunError::RanAmok)
}

/// Puts the subroutine into kseg1, and then calls the subroutine.
pub fn call_raw(subroutine: &crate::mips::Subroutine) -> crate::RunResult<()> {
    let mut cpu = Cpu::new();
    let mut kseg1 = [0; 0x10000];
    let subroutine = subroutine.encode();
    kseg1[0..subroutine.len()].copy_from_slice(&subroutine);

    let mut bus = Bus {
        kseg1,
        is_done: false,
    };

    let end_pc = 0xBFC00000 + subroutine.len() as u32;

    for _ in 0..10000 {
        if cpu.registers().read(RegisterType::Pc) == end_pc {
            return Ok(());
        }
        cpu.clock(&mut bus, 1);
    }
    Err(crate::RunError::RanAmok)
}
