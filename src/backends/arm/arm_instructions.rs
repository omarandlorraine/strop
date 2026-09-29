//! module implementing the ARM instructions (which are fully thirty two bit)

use crate::IterationResult;
use unarm::Ins;

/// Represents an ARM instruction (the 32-bit instruction word type)
pub struct ArmInstruction(u32);

impl std::fmt::Display for ArmInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let i = unarm::parse_arm(self.0, 0, &Default::default());
        write!(f, "{}", i.display(&Default::default()))
    }
}

impl std::fmt::Debug for ArmInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let inner: u32 = self.0;
        let i = unarm::parse_arm(self.0, 0, &Default::default());
        let dasm = format!("{}", i.display(&Default::default()));
        write!(f, "{dasm:<82} ; {inner:#010x} {i:?}")
    }
}

impl ArmInstruction {
    /// Parses the instruction word into an instance of `unarm::Ins`.
    pub fn parse(&self) -> Ins {
        unarm::parse_arm(self.0, 0, &Default::default())
    }
    /// Skips this instruction (that is, increments the instruction word until the instruction
    /// discriminant changes)
    pub fn skip_this_instruction(&mut self) -> IterationResult {
        todo!()
    }
    /// Returns the instruction word
    pub fn codepoint(&self) -> u32 {
        self.0
    }
}

impl crate::Instruction for ArmInstruction {
    fn random() -> Self {
        Self(rand::random())
    }
    fn mutate(&mut self) {
        use rand::RngExt;

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u32 = 1 << rand::rng().random_range(0..32);
            self.0 ^= mask;
        } else {
            // could completely change the instruction word to something completely different
            self.0 = rand::random();
        }
        while self.parse() == unarm::Ins::Illegal {
            // rejection sample: in the vanishingly unlikely case we've generated an invalid
            // instruction, just pick a different one.
            self.0 = rand::random();
        }
    }
    fn first() -> Self {
        Self(0)
    }
    fn increment(&mut self) -> crate::IterationResult {
        if let Some(new_instruction) = self.0.checked_add(1) {
            self.0 = new_instruction;
            Ok(())
        } else {
            Err(crate::StepError::End)
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into()
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let a = bytes.first()?;
        let b = bytes.get(1)?;
        let c = bytes.get(2)?;
        let d = bytes.get(3)?;
        Some(Self(u32::from_le_bytes([*a, *b, *c, *d])))
    }
}
