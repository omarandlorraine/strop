//! Module containing definitions for Z80 and 8080 instruction sets

use crate::Instruction;
use crate::SearchCull;

/// Represents a Z80 instruction
#[derive(Clone, Copy)]
pub struct Z80Instruction {
    mc: [u8; 5],
}

impl std::fmt::Debug for Z80Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:#04x} {:#04x} {:#04x} {:#04x} {:#04x}",
            self.mc[0], self.mc[1], self.mc[2], self.mc[3], self.mc[4]
        )
    }
}

impl Z80Instruction {
    /// Constructs a new Z80Instruction from five bytes.
    pub fn new(mc: [u8; 5]) -> Self {
        Self { mc }
    }

    fn decode(&self) -> dez80::Instruction {
        let encoding = Vec::<_>::from(self.mc);
        dez80::Instruction::decode_one(&mut encoding.as_slice()).unwrap()
    }
}

impl Instruction for Z80Instruction {
    fn random() -> Self {
        use rand::random;
        Self { mc: random() }
    }

    fn mutate(self) -> Self {
        use rand::random;
        use rand::Rng;

        let mut copy = self;

        let offset = rand::thread_rng().gen_range(0..5);
        if random() {
            // try flipping a bit at random
            let bit = rand::thread_rng().gen_range(0..8);
            copy.mc[offset] ^= 1 << bit;
        } else {
            // try straight-up replacing the byte for another
            copy.mc[offset] = random();
        }

        copy
    }

    fn encode(self) -> Vec<u8> {
        self.decode().to_bytes()
    }

    fn first() -> Self {
        Self {
            mc: [0, 0, 0, 0, 0],
        }
    }

    fn increment(&mut self) -> Option<Self> {
        // The length of the instruction
        let offset = self.encode().len();

        for i in offset + 1..5 {
            self.mc[i] = 0;
        }

        fn incr_operand(insn: &mut Z80Instruction, offset: usize) -> bool {
            if let Some(n) = insn.mc[offset].checked_add(1) {
                insn.mc[offset] = n;
                true
            } else {
                insn.mc[offset] = 0;
                if offset > 0 {
                    incr_operand(insn, offset - 1)
                } else {
                    false
                }
            }
        }

        if !incr_operand(self, offset - 1) {
            return None;
        }
        // managed to increment the operand

        // this prefix is ignored if the following byte is in the set below. In other words, there
        // are no valid encodings starting with 0xdd or 0xfd and one of the following bytes.
        // So let's skip it.
        while matches!(self.mc[0], 0xdd | 0xfd)
            && matches!(
                self.mc[1],
                0x00..=0x18
                    | 0x1a..=0x20
                    | 0x27..=0x28
                    | 0x2f..=0x33
                    | 0x37..=0x38
                    | 0x3a..=0x43
                    | 0x47..=0x4b
                    | 0x4f..=0x53
                    | 0x57..=0x5b
                    | 0x5f
                    | 0x70
                    | 0x76
                    | 0x78..=0x7b
                    | 0x7f..=0x83
                    | 0x87..=0x8b
                    | 0x8f..=0x93
                    | 0x97..=0x9b
                    | 0x9f..=0xa3
                    | 0xa7..=0xab
                    | 0xaf..=0xb3
                    | 0xb7..=0xbb
                    | 0xbf..=0xca
                    | 0xcc..=0xe0
                    | 0xe2
                    | 0xe4
                    | 0xe6..=0xe8
                    | 0xea..=0xf8
                    | 0xfa..=0xff
            )
        {
            incr_operand(self, 1);
        }

        // the ED prefix avails the Z80 of the RETI and RETN opcodes, but they are aliased over
        // several encodings. These unnecessary duplicates are skipped. So are other illegal
        // instructions in the ED prefix. And block instructions are also skipped; I'm having
        // trouble with the emulator executing these.
        while self.mc[0] == 0xed {
            match self.mc[1] {
                0x00..=0x3f => self.mc[1] = 0x40, // this range is undefined instructions
                0x55 | 0x5d | 0x65 | 0x6d | 0x75 | 0x7d => {
                    // aliases for RETI and RETN
                    incr_operand(self, 1);
                }
                0x77 | 0x7f => {
                    // undefined instructions
                    incr_operand(self, 1);
                }
                0x80..=0xff => {
                    // undefined instructions
                    incr_operand(self, 0);
                }
                _ => break,
            }
        }

        #[cfg(test)]
        {
            let instruction = self.decode();
            assert!(instruction.ignored_prefixes.is_empty(), "{:?}", self);
            assert!(!format!("{}", instruction).contains("invalid prefix"));
            assert!(
                !format!("{}", instruction).starts_with(';'),
                "invalid encoding, {:?}",
                self
            );
        }
        Some(*self)
    }

    fn cull_flow_control(&self) -> SearchCull<Self> {
        let opcode = self.mc[0];
        match opcode {
            0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xc0 | 0xd2 | 0xd4 | 0xda | 0xdc | 0xe2
            | 0xe4 | 0xec | 0xf2 | 0xf4 | 0xfa | 0xfc => {
                SearchCull::<Self>::SkipTo(Some(Self::new([opcode + 1, 0, 0, 0, 0])))
            }
            0xc2..=0xc4 => SearchCull::<Self>::SkipTo(Some(Self::new([0xc5, 0, 0, 0, 0]))),
            0xc7..=0xca => SearchCull::<Self>::SkipTo(Some(Self::new([0xcb, 0, 0, 0, 0]))),
            0xcc..=0xcd => SearchCull::<Self>::SkipTo(Some(Self::new([0xce, 0, 0, 0, 0]))),
            0xcf..=0xd0 => SearchCull::<Self>::SkipTo(Some(Self::new([0xd1, 0, 0, 0, 0]))),
            0xd7..=0xd8 => SearchCull::<Self>::SkipTo(Some(Self::new([0xd9, 0, 0, 0, 0]))),
            0xdd | 0xfd => {
                if self.mc[1] == 0xe9 {
                    SearchCull::<Self>::SkipTo(Some(Self::new([0xdd, 0xea, 0, 0, 0])))
                } else {
                    SearchCull::<Self>::Okay
                }
            }
            0xdf..=0xe0 => SearchCull::<Self>::SkipTo(Some(Self::new([0xe1, 0, 0, 0, 0]))),
            0xe7..=0xea => SearchCull::<Self>::SkipTo(Some(Self::new([0xeb, 0, 0, 0, 0]))),
            0xed => {
                if matches!(
                    self.mc[1],
                    0x4d | 0x45 | 0x55 | 0x5d | 0x65 | 0x6d | 0x75 | 0x7d
                ) {
                    SearchCull::<Self>::SkipTo(Some(Self::new([0xed, self.mc[1] + 1, 0, 0, 0])))
                } else {
                    SearchCull::<Self>::Okay
                }
            }
            0xef..=0xf0 => SearchCull::<Self>::SkipTo(Some(Self::new([0xf1, 0, 0, 0, 0]))),
            0xf7..=0xf8 => SearchCull::<Self>::SkipTo(Some(Self::new([0xf9, 0, 0, 0, 0]))),
            0xff => SearchCull::<Self>::SkipTo(None),
            _ => SearchCull::<Self>::Okay,
        }
    }
}

impl std::fmt::Display for Z80Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let instruction = dez80::Instruction::decode_one(&mut self.encode().as_slice()).unwrap();
        write!(f, "{}", instruction)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn disassembly() {
        use crate::Instruction;
        let mut p = super::Z80Instruction::first();
        while let Some(insn) = p.increment() {
            format!("{}", insn);
        }
    }

    #[test]
    fn invalid_prefixes() {}

    #[test]
    fn instruction_increment() {
        use crate::Instruction;
        let mut p = super::Z80Instruction::first();
        assert!(p.increment().is_some());
        assert!(p.increment().is_some());
        assert!(p.increment().is_some());
        assert!(p.increment().is_some());
    }

    #[test]
    fn bruteforce_search() {
        use crate::Instruction;
        let mut p = super::Z80Instruction::first();
        p.increment().unwrap();
        p.increment().unwrap();
        p.increment().unwrap();
    }

    //#[test]
    fn lengths() {
        use super::Z80Instruction;
        use crate::z80::emulators::Z80;
        use crate::BruteForceSearch;
        use crate::Emulator;
        use crate::Instruction;
        use crate::SearchAlgorithm;

        // If it's not a flow-control instruction, then we can check the length of the instruction
        // against the program counter in the emulator
        for cand in BruteForceSearch::<Z80Instruction>::new()
            .limit_length(1)
            .no_flow_control()
            .iter()
        {
            let mut emu = Z80::default();
            emu.run(0, &cand);
            let insn = cand.instructions[0];
            assert_eq!(
                cand.encode().len(),
                emu.get_pc().into(),
                "{}; {:?} {:?}",
                insn,
                insn,
                insn.encode()
            );
        }
    }

    #[test]
    fn is_valid_encoding() {
        use super::Z80Instruction;
        use crate::Instruction;

        let mut insn = Z80Instruction::first();

        while insn.increment().is_some() {
            let dasm = format!("{}", insn);
            assert!(!dasm.contains("invalid prefix"), "{} {:?}", dasm, insn)
        }
    }

    #[test]
    fn cull_flow_control() {
        use super::Z80Instruction;
        use crate::Instruction;

        let mut insn = Z80Instruction::first();
        let opcodes = vec!["CALL", "RET", "RST", "JP", "JR", "DJNZ"];

        while insn.increment().is_some() {
            let dasm = format!("{}", insn);

            for opcode in &opcodes {
                if dasm.starts_with(opcode) {
                    assert!(
                        !insn.cull_flow_control().is_okay(),
                        "{} {:?} should be marked as a flow control instruction",
                        dasm,
                        insn
                    )
                }
            }

            if insn.cull_flow_control().is_okay() {
                for opcode in &opcodes {
                    assert!(
                        !dasm.starts_with(opcode),
                        "{} should not be marked as a flow control instruction",
                        dasm
                    )
                }
            } else if let Some(s) = insn.cull_flow_control().suggestion() {
                assert!(s.cull_flow_control().is_okay())
            }
        }
    }
}
