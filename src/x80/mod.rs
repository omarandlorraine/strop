//! Common code for Intel 8080-like backends, including z80 and sm83
use crate::ShouldReturn;
use crate::Step;
pub mod data;
pub mod sdcccall1;

/// Trait describing the workings of an emulator. This trait is enough to be able to call a
/// function in a few different ways.
pub trait EmuInterface {
    /// Sets the emulator's accumulator to some value
    fn set_a(&mut self, val: u8);

    /// Sets the emulator's B register to some value
    fn set_b(&mut self, val: u8);

    /// Sets the emulator's C register to some value
    fn set_c(&mut self, val: u8);

    /// Sets the emulator's C register to some value
    fn set_d(&mut self, val: u8);

    /// Sets the emulator's C register to some value
    fn set_e(&mut self, val: u8);

    /// Sets the emulator's B register to some value
    fn set_h(&mut self, val: u8);

    /// Sets the emulator's C register to some value
    fn set_l(&mut self, val: u8);

    /// Sets the emulator's BC register to some value
    fn set_bc(&mut self, val: u16) {
        let [hi, lo] = val.to_be_bytes();
        self.set_b(hi);
        self.set_c(lo);
    }

    /// Sets the emulator's HL register to some value
    fn set_hl(&mut self, val: u16) {
        let [hi, lo] = val.to_be_bytes();
        self.set_h(hi);
        self.set_l(lo);
    }

    /// Returns the value of the accumulator
    fn get_a(&self) -> u8;

    /// Returns the value of the emulator's HL register
    fn get_hl(&self) -> u16;

    /// Returns the value of the emulator's program counter
    fn get_pc(&self) -> u16;

    /// Returns the value of the emulator's stack pointer
    fn get_sp(&self) -> u16;

    /// Returns the value of the emulator's H register
    fn get_h(&self) -> u8 {
        self.get_hl().to_be_bytes()[0]
    }

    /// Returns the value of the emulator's L register
    fn get_l(&self) -> u8 {
        self.get_hl().to_be_bytes()[1]
    }

    /// Pushes a word onto the stack
    fn push(&mut self, val: u16);

    /// Pops a word off the stack and returns it
    fn pop(&mut self) -> u16;

    /// Returns a new emulator
    fn new() -> Self;

    /// Writes a subroutine to memory and then calls it
    fn call(&mut self, seq: Vec<u8>) -> crate::RunResult<()>;
}

/// Associates an Instruction type with an Emulator type
pub trait X80: Step + ShouldReturn + Clone {
    /// The type of emulator used for this instruction set
    type Emulator: EmuInterface;

    /// returns a reference to an InstructionData
    fn decode(&self) -> &'static data::InstructionData;

    /// advances to the next opcode;
    fn next_opcode(&mut self) -> crate::IterationResult;
}
