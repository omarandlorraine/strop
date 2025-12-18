//! Common code for Intel 8080-like backends, including z80 and sm83
pub mod data;
pub mod sdcccall1;
pub use sdcccall1::SdccCall1;

/// Trait describing the workings of an emulator. This trait is enough to be able to call a
/// function in a few different ways.
pub trait EmuInterface: Default {
    /// Sets the emulator's accumulator to some value
    fn set_a(&mut self, val: u8);

    /// Sets the emulator's B register to some value
    fn set_b(&mut self, val: u8);

    /// Sets the emulator's C register to some value
    fn set_c(&mut self, val: u8);

    /// Sets the emulator's D register to some value
    fn set_d(&mut self, val: u8);

    /// Sets the emulator's E register to some value
    fn set_e(&mut self, val: u8);

    /// Sets the emulator's H register to some value
    fn set_h(&mut self, val: u8);

    /// Sets the emulator's L register to some value
    fn set_l(&mut self, val: u8);

    /// Sets the emulator's DE register to some value
    fn set_de(&mut self, val: u16) {
        let [hi, lo] = val.to_be_bytes();
        self.set_d(hi);
        self.set_e(lo);
    }

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

    /// Returns the value of the B register
    fn get_b(&self) -> u8;

    /// Returns the value of the C register
    fn get_c(&self) -> u8;

    /// Returns the value of the D register
    fn get_d(&self) -> u8;

    /// Returns the value of the E register
    fn get_e(&self) -> u8;

    /// Returns the value of the BC register
    fn get_bc(&self) -> u16 {
        u16::from_be_bytes([self.get_b(), self.get_c()])
    }

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
    #[allow(dead_code)]
    fn pop(&mut self) -> u16;

    /// Writes a subroutine to memory and then calls it
    fn call(&mut self, seq: Vec<u8>) -> crate::RunResult<()>;

    /// Writes a byte to the memory
    fn poke(&mut self, addr: u16, val: u8);

    /// Reads a byte from the memory
    #[allow(dead_code)]
    fn peek(&mut self, addr: u16) -> u8;

    /// Executes one intruction
    fn single_step(&mut self) -> crate::RunResult<()>;
}

/// Associates an Instruction type with an Emulator type
pub trait X80: crate::Instruction + Sized {
    /// The type of emulator used for this instruction set
    type Emulator: EmuInterface;

    /// returns a reference to an InstructionData
    fn decode(&self) -> &'static data::InstructionData;

    /// Increments the opcode part of the instruction, the opcode and any prefixes.
    fn next_opcode(&mut self) -> crate::IterationResult;

    /// return the fixup to make this return
    fn make_return(&self) -> crate::StaticAnalysis<Self>;

    /// return the fixup to make this instruction pure
    fn make_pure(&self) -> crate::StaticAnalysis<Self> {
        crate::static_analysis::Fixup::<Self>::check(
            !self.decode().is_impure(),
            "ImpureInstruction",
            Self::next_opcode,
            0,
        )
    }

    // returns the length of the instruction, i.e.  the number of prefix bytes plus the opcode.
    fn instruction_length(&self) -> usize;
}

/// Associates an Instruction type with a type that can run the subroutines, passing arguments in
/// accordance with the SDCCCALL1 calling convention
pub trait SdccCallable: X80 {
    /// the type of emulator used to call __sdcccall(1) functions in this instruction set.
    type Runner: sdcccall1::SdccCall1PushPop
        + crate::test::GetReturnValues
        + crate::test::TakeParameters
        + EmuInterface
        + Default;
}

#[cfg(test)]
pub(crate) mod tests;
