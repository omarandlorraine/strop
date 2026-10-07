//! module implementing the ARM instructions (which are fully thirty two bit)

use crate::IterationResult;
use unarm::Ins;

/// Represents a Thumb instruction (being Thumb2, it may have either one or two halfwords)
pub struct ThumbInstruction {
    first: u16,
    second: u16,
}

impl std::fmt::Display for ThumbInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.parse().display(&Default::default()))
    }
}

impl std::fmt::Debug for ThumbInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{} ; {:#010x}",
            self.parse().display(&Default::default()),
            self.codepoint()
        )
    }
}

impl ThumbInstruction {
    /// Parses the instruction word into an instance of `unarm::Ins`.
    pub fn parse(&self) -> Ins {
        let (ins, _size) = unarm::parse_thumb(self.codepoint(), 0, &Default::default());
        ins
    }
    /// Skips this instruction (that is, increments the instruction word until the instruction
    /// discriminant changes)
    pub fn skip_this_instruction(&mut self) -> IterationResult {
        todo!()
    }
    /// Returns `true` iff the Thumb2 instruction uses thirty two bits
    pub fn is_32bit(&self) -> bool {
        let first = self.first as u32;
        let second = self.second as u32;
        let codepoint = first | (second << 16);
        let (_ins, size) = unarm::parse_thumb(codepoint, 0, &Default::default());
        size > 2
    }
    /// Returns the instruction word
    pub fn codepoint(&self) -> u32 {
        let first = self.first as u32;
        let second = self.second as u32;
        if self.is_32bit() {
            first | (second << 16)
        } else {
            first
        }
    }
}

impl crate::Instruction for ThumbInstruction {
    fn random() -> Self {
        Self {
            first: rand::random(),
            second: rand::random(),
        }
    }
    fn mutate(&mut self) {
        use rand::RngExt;

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u16 = 1 << rand::rng().random_range(0..32);
            if rand::random() {
                self.first ^= mask;
            } else {
                self.second ^= mask;
            }
        } else {
            // could completely change the instruction word to something completely different
            if rand::random() {
                self.first = rand::random();
            } else {
                self.second = rand::random();
            }
        }
        while self.parse() == unarm::Ins::Illegal {
            // rejection sample: in the vanishingly unlikely case we've generated an invalid
            // instruction, just pick a different one.
            self.first = rand::random();
        }
    }
    fn first() -> Self {
        Self {
            first: 0,
            second: 0,
        }
    }
    fn increment(&mut self) -> crate::IterationResult {
        if self.is_32bit()
            && let Some(new_second) = self.second.checked_add(1)
        {
            self.second = new_second;
            return Ok(());
        }
        // else fall through to incrementing .first
        if let Some(new_first) = self.first.checked_add(1) {
            self.first = new_first;
            self.second = 0;
            Ok(())
        } else {
            Err(crate::StepError::End)
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        if self.is_32bit() {
            let first = self.first.to_le_bytes();
            let second = self.second.to_le_bytes();
            vec![first[0], first[1], second[0], second[1]]
        } else {
            self.first.to_le_bytes().into()
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let a = bytes.first()?;
        let b = bytes.get(1)?;
        let first = u16::from_le_bytes([*a, *b]);
        let s = Self { first, second: 0 };
        if s.is_32bit() {
            let c = bytes.get(2)?;
            let d = bytes.get(3)?;
            Some(Self {
                first,
                second: u16::from_le_bytes([*c, *d]),
            })
        } else {
            Some(s)
        }
    }
}
