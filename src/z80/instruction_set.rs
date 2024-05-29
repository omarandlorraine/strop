//! Module containing definitions for Z80 and 8080 instruction sets

use crate::Instruction;

/// Represents a Z80 instruction
#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Z80Instruction([u8; 5]);

impl std::fmt::Debug for Z80Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:#04x} {:#04x} {:#04x} {:#04x} {:#04x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4]
        )
    }
}

impl Z80Instruction {
    /// Constructs a new Z80Instruction from five bytes.
    pub const fn new(mc: [u8; 5]) -> Self {
        Self(mc)
    }

    fn decode(&self) -> dez80::Instruction {
        let encoding = Vec::<_>::from(self.0);
        let e = dez80::Instruction::decode_one(&mut encoding.as_slice());
        match e {
            Ok(e) => e,
            Err(e) => panic!(
                "couldn't encode {:?}: {:?}",
                self.0
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<String>>()
                    .join(" "),
                e
            ),
        }
    }

    fn invalid(&self) -> Option<usize> {
        // if the 5 bytes don't encode a valid Z80 instruction, then it could be because the opcode
        // is unknown, or maybe the prefix bytes aren't right, or whatever, so we need to return a
        // offset to mutate the instruction at.

        match self.0[0] {
            0xcb => match self.0[1] {
                0x30..=0x37 => Some(1),
                _ => None,
            },
            0xdd | 0xfd => match self.0[1] {
                0x00..=0x08 => Some(1),
                0x0a..=0x18 => Some(1),
                0x1a..=0x1b => Some(1),
                0x1c..=0x20 => Some(1),
                0x27..=0x28 => Some(1),
                0x2f..=0x33 => Some(1),
                0x37..=0x38 => Some(1),
                0x3a..=0x43 => Some(1),
                0x47..=0x4b => Some(1),
                0x4f..=0x53 => Some(1),
                0x57..=0x5b => Some(1),
                0x5f => Some(1),
                0x76..=0x7b => Some(1),
                0x7f..=0x83 => Some(1),
                0x87..=0x8b => Some(1),
                0x8f..=0x93 => Some(1),
                0x97..=0x9b => Some(1),
                0x9f..=0xa3 => Some(1),
                0xa7..=0xab => Some(1),
                0xaf..=0xb3 => Some(1),
                0xb7..=0xbb => Some(1),
                0xbf..=0xca => Some(1),
                0xcc..=0xe1 => Some(1),
                0xe2 | 0xe4 => Some(1),
                0xe6..=0xe8 => Some(1),
                0xea..=0xf8 => Some(1),
                0xfa..=0xff => Some(1),
                _ => None,
            },
            0xed => match self.0[1] {
                0x00..=0x3f => Some(1),
                0x4c | 0x4e => Some(1),
                0x55..=0x56 => Some(1),
                0x5c..=0x5d => Some(1),
                0x64..=0x66 => Some(1),
                0x6c..=0x6e => Some(1),
                0x74..=0x77 => Some(1),
                0x7c..=0x9f => Some(1),
                0xa4..=0xa7 => Some(1),
                0xac..=0xaf => Some(1),
                0xb4..=0xb7 => Some(1),
                0xbc..=0xff => Some(1),
                _ => None,
            },
            _ => None,
        }
    }

    /// Gets the next instruction by incrementing the first byte (which would normally be the
    /// opcode, but could also be a prefix).
    pub fn increment_opcode(&self) -> Option<Self> {
        Some(Z80Instruction([self.0[0].checked_add(1)?, 0, 0, 0, 0]))
    }

    /// Returns true if the opcode is present on the Intel 8080
    pub fn opcode_present_on_8080(&self) -> bool {
        // excludes the opcodes:
        //  - `ex af,af'` and `exx`
        //  - the relative jumps
        //  - `djnz`
        !matches!(
            self.0[0],
            0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xd9 | 0xcb | 0xed | 0xdd | 0xfd
        )
    }

    /// Returns true if the opcode is present on the SM83
    pub fn opcode_present_on_sm83(&self) -> bool {
        // this information is sourced from https://gbdev.io/pandocs/CPU_Comparison_with_Z80.html
        // excludes the opcodes:
        // 08	ex af,af
        // 10	djnz pc+dd
        // 22	ld (nn),hl
        // 2A	ld hl,(nn)
        // 32	ld (nn),a
        // 3A	ld a,(nn)
        // D3	out (n),a
        // D9	exx
        // DB	in a,(n)
        // DD	<IX> prefix
        // E0	ret po
        // E2	jp po,nn
        // E3	ex (sp),hl
        // E4	call p0,nn
        // E8	ret pe
        // EA	jp pe,nn
        // EB	ex de,hl
        // EC	call pe,nn
        // ED	<prefix>
        // F0	ret p
        // F2	jp p,nn
        // F4	call p,nn
        // F8	ret m
        // FA	jp m,nn
        // FC	call m,nn
        // FD	<IY> prefix
        // CB 3X	sll <something>
        match self.0[0] {
            0x08 | 0x10 | 0x22 | 0x2a | 0x32 | 0x3a | 0xd3 | 0xd9 | 0xdb | 0xdd | 0xe0 | 0xe2
            | 0xe3 | 0xe4 | 0xe8 | 0xea | 0xeb | 0xec | 0xed | 0xf0 | 0xf2 | 0xf4 | 0xf8 | 0xfa
            | 0xfc | 0xfd => false,
            0xcb => self.0[1] & 0xf0 != 0x30,
            _ => true,
        }
    }

    /// returns true if the instruction is documented by Zilog (i.e., it is not an undocumented
    /// instruction)
    pub fn opcode_is_documented(&self) -> bool {
        #[allow(clippy::collapsible_if)]
        // this information is sourced from http://www.z80.info/z80undoc.htm
        if matches!(self.0[0], 0xdd | 0xfd) {
            if matches!(
                self.0[1],
                0x24..=0x26 | 0x2c..=0x2e | 0x44..=0x45 | 0x4c | 0x4d | 0x54 | 0x55 | 0x5c | 0x5d
                | 0x60..=0x65 | 0x67..=0x6d | 0x6f | 0x7c | 0x7d | 0x84 | 0x85 | 0x8c | 0x8d | 0x94 | 0x95
                | 0x9c | 0x9d | 0xa4 | 0xa5 | 0xac | 0xad | 0xb4 | 0xb5 | 0xbc | 0xbd
            ) {
                // it's an instruction which uses the individual halves of the IX and IY register
                // these instructions trap on an HD64180 IIRC
                return false;
            }
        }
        // TODO: There's more to come here.
        true
    }
}

impl Instruction for Z80Instruction {
    fn random() -> Self {
        use rand::random;
        Self(random())
    }

    fn mutate(&mut self) {
        use rand::random;
        use rand::Rng;

        let offset = rand::thread_rng().gen_range(0..5);
        if random() {
            // try flipping a bit at random
            let bit = rand::thread_rng().gen_range(0..8);
            self.0[offset] ^= 1 << bit;
        } else {
            // try straight-up replacing the byte for another
            self.0[offset] = random();
        }

        while let Some(invalid) = self.invalid() {
            self.0[invalid] = random();
        }
    }

    fn encode(self) -> Vec<u8> {
        self.decode().to_bytes()
    }

    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn increment(&mut self) -> Option<Self> {
        if self.0[0] == 0xff {
            // There's no way to increment this.
            return None;
        }

        fn incr_operand(insn: &mut Z80Instruction, offset: usize) -> bool {
            if let Some(n) = insn.0[offset].checked_add(1) {
                insn.0[offset] = n;
                true
            } else {
                insn.0[offset] = 0;
                incr_operand(insn, offset - 1)
            }
        }

        // The length of the instruction
        let offset = self.encode().len();
        incr_operand(self, offset - 1);
        while let Some(offs) = self.invalid() {
            incr_operand(self, offs);
        }

        Some(*self)
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
        let mut previous = super::Z80Instruction::first();
        let mut next = previous;
        while let Some(n) = next.increment() {
            if !n.decode().ignored_prefixes.is_empty() {
                let mut to = n;
                let mut tof = format!("{:?}", to);

                while !to.decode().ignored_prefixes.is_empty() {
                    tof = format!("{:?}", to);
                    to.increment();
                }
                panic!("{:?} to {} have ignored prefixes", n, tof);
            }

            assert!(
                !format!("{}", n).starts_with(';'),
                "invalid encoding, {:?}",
                n
            );
            assert!(n > previous);
            previous = next
        }
    }

    #[test]
    fn bruteforce_search() {
        use crate::Instruction;
        let mut p = super::Z80Instruction::first();
        p.increment().unwrap();
        p.increment().unwrap();
        p.increment().unwrap();
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
}
