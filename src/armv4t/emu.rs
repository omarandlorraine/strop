use armv4t_emu::{Cpu, ExampleMem};

/// A basic emulator
#[derive(Default)]
pub struct Emulator {
    /// The emulated system's memory
    pub mem: ExampleMem,
    /// The emulated system's CPU
    pub cpu: Cpu,
}

impl std::fmt::Debug for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.cpu)
    }
}
