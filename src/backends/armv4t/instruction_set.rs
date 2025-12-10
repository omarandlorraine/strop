//! This module defines how an ARMv4T machine code instruction works

/// Represents an ARMv4T machine code instruction.
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Instruction(pub(crate) u32);

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let i = unarm::parse_arm(self.0, 0, &Default::default());
        write!(f, "{}", i.display(&Default::default()))
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let inner: u32 = self.0;
        let i = unarm::parse_arm(self.0, 0, &Default::default());
        let dasm = format!("{}", i.display(&Default::default()));
        write!(f, "{dasm:<82} ; {inner:#010x} {i:?}")
    }
}

impl crate::Instruction for Instruction {
    fn first() -> Self {
        Self(0)
    }

    fn random() -> Self {
        Self(rand::random())
    }

    fn mutate(&mut self) {
        use rand::Rng;

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u32 = 1 << rand::rng().random_range(0..32);
            self.0 ^= mask;
        } else {
            // could completely change the instruction word to something completely different
            self.0 = rand::random();
        }
        while self.exclude_bad_instructions().is_err() {
            // rejection sample: in the vanishingly unlikely case we've generated an invalid
            // instruction, just pick a different one.
            self.0 = rand::random();
        }
    }

    fn increment(&mut self) -> crate::IterationResult {
        if self.0 == 0xffffffff {
            Err(crate::StepError::End)
        } else {
            self.0 += 1;
            self.exclude_bad_instructions()?;
            Ok(())
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self(u32::from_le_bytes([
            *bytes.first()?,
            *bytes.get(1)?,
            *bytes.get(2)?,
            *bytes.get(3)?,
        ])))
    }
}

impl Instruction {
    /// Returns the fixup that makes this a `bx lr` instruction
    pub fn make_bx_lr(&self) -> crate::StaticAnalysis<Self> {
        const INSN: u32 = 0xe12fff1e;
        crate::Fixup::<Self>::check(
            self.0 == INSN,
            "DoesNotReturn",
            |i| {
                if i.0 <= INSN {
                    i.0 = INSN;
                    Ok(())
                } else {
                    Err(crate::StepError::End)
                }
            },
            0,
        )
    }

    /// Decodes the instruction
    pub fn decode(&self) -> unarm::Ins {
        unarm::parse_arm(self.0, 0, &Default::default())
    }

    /// Increments the instruction to the first instruction having the next condition code
    pub fn increment_condition(&mut self) -> crate::IterationResult {
        if self.0 >= 0xf000_0000 {
            Err(crate::StepError::End)
        } else {
            self.0 |= 0x0fff_ffff;
            self.0 += 1;
            Ok(())
        }
    }

    /// Increments the instruction to the first instruction having the next horrid nybble
    pub fn increment_horrid_nybble(&mut self) -> crate::IterationResult {
        self.0 |= 0x0000_000f;
        if let Some(n) = self.0.checked_add(1) {
            self.0 = n;
            Ok(())
        } else {
            Err(crate::StepError::End)
        }
    }

    /// excludes bad instructions from consideration
    fn exclude_bad_instructions(&mut self) -> crate::IterationResult {
        use unarm::Ins;
        loop {
            match self.decode() {
                Ins::Illegal => self.0 += 1,
                Ins::Ldrh { .. } | Ins::Ldrsh { .. } | Ins::Ldrsb { .. }
                    if self.0 & 0x0040_0000 != 0 =>
                {
                    self.0 += 1;
                }
                Ins::Strh { .. } | Ins::Strd { .. } | Ins::Ldrd { .. }
                    if self.0 & 0x0080_0000 != 0 =>
                {
                    self.0 += 1;
                }
                _ => break,
            }
        }
        Ok(())
    }
}
