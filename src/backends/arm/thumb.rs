//! Represents a Thumb instruction, as is executable on most ARM processors.

use super::Variant;
use crate::IterationResult;

#[derive(Debug, Clone)]
pub struct BadInstructionRange(pub std::ops::RangeInclusive<u16>);

impl From<std::ops::RangeInclusive<u16>> for BadInstructionRange {
    fn from(r: std::ops::RangeInclusive<u16>) -> Self {
        Self(r)
    }
}

impl BadInstructionRange {
    pub fn contains(&self, insn: u16) -> bool {
        self.0.contains(&insn)
    }
    pub fn next_good_instruction(&self) -> Result<u16, crate::StepError> {
        if let Some(i) = self.0.end().checked_add(1) {
            Ok(i)
        } else {
            Err(crate::StepError::End)
        }
    }
}

pub mod bad_instruction_ranges {
    use super::BadInstructionRange;

    // these ones seem to panic in armagnac, so we can't have strop generate these instructions.
    pub const UNSUPPORTED_BY_ARMAGNAC: [BadInstructionRange; 2] = [
        BadInstructionRange(0x1000..=0x103f),
        BadInstructionRange(0xbf40..=0xbf40),
    ];

    // these ones get disassembled to `<illegal>`.
    pub const ILLEGAL: [BadInstructionRange; 27] = [
        BadInstructionRange(0xb100..=0xb1ff),
        BadInstructionRange(0xb300..=0xb4ff),
        BadInstructionRange(0xb600..=0xb64f),
        BadInstructionRange(0xb651..=0xb657),
        BadInstructionRange(0xb659..=0xb65f),
        BadInstructionRange(0xb668..=0xb66f),
        BadInstructionRange(0xb678..=0xba7f),
        BadInstructionRange(0xba80..=0xbabf),
        BadInstructionRange(0xbb00..=0xbc00),
        BadInstructionRange(0xbf00..=0xbfff),
        BadInstructionRange(0xc000..=0xc000),
        BadInstructionRange(0xc100..=0xc100),
        BadInstructionRange(0xc200..=0xc200),
        BadInstructionRange(0xc300..=0xc300),
        BadInstructionRange(0xc400..=0xc400),
        BadInstructionRange(0xc500..=0xc500),
        BadInstructionRange(0xc600..=0xc600),
        BadInstructionRange(0xc700..=0xc700),
        BadInstructionRange(0xc800..=0xc800),
        BadInstructionRange(0xc900..=0xc900),
        BadInstructionRange(0xca00..=0xca00),
        BadInstructionRange(0xcb00..=0xcb00),
        BadInstructionRange(0xcc00..=0xcc00),
        BadInstructionRange(0xcd00..=0xcd00),
        BadInstructionRange(0xce00..=0xce00),
        BadInstructionRange(0xcf00..=0xcf00),
        BadInstructionRange(0xe800..=0xffff),
    ];
}

/// Represents a Thumb instruction
#[derive(Clone)]
pub struct ThumbInstruction<V: Variant> {
    instruction: u16,
    _phantom: std::marker::PhantomData<V>,
}

impl<V: Variant> std::fmt::Display for ThumbInstruction<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let i = unarm::parse_thumb(self.instruction.into(), 0, &Default::default()).0;
        write!(f, "{}", i.display(&Default::default()))
    }
}

impl<V: Variant> std::fmt::Debug for ThumbInstruction<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let inner: u16 = self.instruction;
        let i = unarm::parse_thumb(inner.into(), 0, &Default::default()).0;
        let dasm = format!("{}", i.display(&Default::default()));
        write!(f, "{dasm:<82} ; {inner:#06x} {i:?}")
    }
}

impl<V: Variant> ThumbInstruction<V> {
    /// Checks the instruction against all the BadInstructionRanges
    pub fn check(&self, rng: &[BadInstructionRange]) -> Result<(), BadInstructionRange> {
        if let Some(r) = rng.iter().find(|r| r.contains(self.instruction)) {
            Err(r.clone())
        } else {
            Ok(())
        }
    }

    /// Returns the fixup that makes this a `bx lr` instruction
    pub fn make_bx_lr(&self) -> crate::StaticAnalysis<Self> {
        const INSN: u16 = 0x4770;
        crate::Fixup::<Self>::check(
            self.instruction == INSN,
            "DoesNotReturn",
            |i| {
                if i.instruction <= INSN {
                    i.instruction = INSN;
                    Ok(())
                } else {
                    Err(crate::StepError::End)
                }
            },
            0,
        )
    }

    #[cfg(test)]
    fn single_step(&self) -> Result<Option<armagnac::core::Event>, armagnac::core::RunError> {
        use crate::Instruction;
        use armagnac::core::Emulator;
        use armagnac::core::RunOptions;

        let mut proc = V::proc();
        proc.map(0x1000, &self.to_bytes()).unwrap();
        proc.set_pc(0x1000);
        proc.run(RunOptions::new().gas(1))
    }
}

impl<V: Variant> crate::Instruction for ThumbInstruction<V> {
    fn random() -> Self {
        Self {
            instruction: rand::random(),
            _phantom: Default::default(),
        }
    }

    fn mutate(&mut self) {
        use rand::Rng;

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u16 = 1 << rand::rng().random_range(0..16);
            self.instruction ^= mask;
        } else {
            // could completely change the instruction word to something completely different
            self.instruction = rand::random();
        }
        while V::check(self).is_err() {
            self.instruction = rand::random();
        }
    }

    fn first() -> Self {
        Self {
            instruction: 0,
            _phantom: Default::default(),
        }
    }

    fn increment(&mut self) -> IterationResult {
        if self.instruction == 0xffff {
            return Err(crate::StepError::End);
        } else {
            self.instruction += 1;
        }
        while let Err(e) = V::check(self) {
            self.instruction = e.next_good_instruction()?;
        }
        Ok(())
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.instruction.to_le_bytes().into()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let le_bytes = [*bytes.first()?, *bytes.get(1)?];
        Some(Self {
            instruction: u16::from_le_bytes(le_bytes),
            _phantom: Default::default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::ThumbInstruction;
    use crate::Instruction;
    use crate::backends::arm::Armv6m;
    use crate::backends::arm::Armv7em;
    use crate::backends::arm::Armv7m;
    use crate::backends::arm::Armv8m;
    use crate::backends::arm::Variant;

    #[test]
    fn first_few() {
        let mut i: ThumbInstruction<Armv6m> = ThumbInstruction::first();
        for _ in 0..100 {
            println!("{i:?}");
            i.increment().unwrap();
        }
    }

    #[test]
    fn illegal() {
        fn c<V: Variant + Clone>() {
            let mut i = ThumbInstruction::<V>::first();
            loop {
                if format!("{i}").contains("illegal") {
                    let mut j = i.clone();
                    while format!("{j}").contains("illegal") {
                        if j.increment().is_err() {
                            break;
                        }
                    }
                    panic!(
                        "{} missing illegal instruction range {:#06x}..{:#06x},",
                        V::name(),
                        i.instruction,
                        j.instruction
                    );
                }
                if i.increment().is_err() {
                    break;
                }
            }
        }
        c::<Armv6m>();
        c::<Armv7m>();
        c::<Armv7em>();
        c::<Armv8m>();
    }

    #[test]
    fn disassemblies_unique() {
        crate::generic_unit_tests::disassemblies_unique::<ThumbInstruction<Armv6m>>(
            ThumbInstruction::first(),
            None,
        );
        crate::generic_unit_tests::disassemblies_unique::<ThumbInstruction<Armv7m>>(
            ThumbInstruction::first(),
            None,
        );
        crate::generic_unit_tests::disassemblies_unique::<ThumbInstruction<Armv7em>>(
            ThumbInstruction::first(),
            None,
        );
        crate::generic_unit_tests::disassemblies_unique::<ThumbInstruction<Armv8m>>(
            ThumbInstruction::first(),
            None,
        );
    }

    #[test]
    fn sanity_checks() {
        crate::generic_unit_tests::sanity_checks::<ThumbInstruction<Armv6m>>();
        crate::generic_unit_tests::sanity_checks::<ThumbInstruction<Armv7m>>();
        crate::generic_unit_tests::sanity_checks::<ThumbInstruction<Armv7em>>();
        crate::generic_unit_tests::sanity_checks::<ThumbInstruction<Armv8m>>();
    }

    #[test]
    fn single_step() {
        fn c<V: Variant + Clone>() {
            let mut i = ThumbInstruction::<V>::first();
            loop {
                println!("{:?}", i);
                if let Err(e) = i.single_step()
                    && matches!(e, armagnac::core::RunError::Unpredictable)
                {
                    // we've found an instruction which doesn't run. Find the next one which does
                    // run.
                    let mut j = i.clone();
                    loop {
                        if j.increment().is_ok() {
                            match j.single_step() {
                                Ok(_) => continue,
                                Err(e) if matches!(e, armagnac::core::RunError::Unpredictable) => {
                                    panic!(
                                        "{} missing BadInstructionRange({:#06x}..={:#06x}), {:?}",
                                        V::name(),
                                        i.instruction,
                                        j.instruction - 1,
                                        e
                                    )
                                }
                                _ => continue,
                            }
                        } else {
                            panic!(
                                "{} missing BadInstructionRange({:#06x}..=0xffff), {:?}",
                                V::name(),
                                i.instruction,
                                e
                            );
                        }
                    }
                }
                if i.increment().is_err() {
                    break;
                }
            }
        }
        //c::<Armv6m>();
        //c::<Armv7m>();
        //c::<Armv7em>();
        c::<Armv8m>();
    }

    #[test]
    fn identity() {
        use crate::Callable;
        use crate::Traverse;
        use crate::backends::arm::Aapcs32;

        let identity_function = Aapcs32::<Armv6m, u32, u32>::from_bytes(&[0x70, 0x47]).unwrap();
        println!("{identity_function}");
        assert_eq!(42, identity_function.call(42).unwrap());
    }
}
