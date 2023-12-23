//! Module containing definitions for Z80 and 8080 instruction sets

use crate::Instruction;
use crate::InstructionSet;

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
    fn i8080_fixup(&mut self) {
        // Some opcodes don't exist on the 8080, so this function changes them
        self.mc[0] = match self.mc[0] {
            0x08 => 0x09, // ex af, af'
            0x10 => 0x11, // djnz
            0x18 => 0x19, // jr off
            0x20 => 0x21, // jr nz,off
            0x28 => 0x29, // jr z,off
            0x30 => 0x31, // jr nc,off
            0x38 => 0x39, // jr c,off
            0xd9 => 0xda, // exx

            // and the prefixes:
            0xcb => 0xcc,
            0xed => 0xee,
            0xdd => 0xde,
            0xfd => 0xfe,
            opcode => opcode,
        }
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
        let encoding = Vec::<_>::from(self.mc);
        // We now have the bytes in the right order, but there's possibly garbage bytes at the end
        // of the Vec<u8>, because there's variable-length instruction stored in a [u8; 5]. So
        // therefore I'm going to use the dez80 crate to decode the instruction, and then re-encode
        // it. The re-encoding of course will not include these garbage bytes.

        let instruction = dez80::Instruction::decode_one(&mut encoding.as_slice()).unwrap();
        instruction.to_bytes()
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

        #[cfg(test)]
        {
            let instruction =
                dez80::Instruction::decode_one(&mut self.encode().as_slice()).unwrap();
            assert!(instruction.ignored_prefixes.is_empty(), "{:?}", self);
        }

        // managed to increment the operand
        if incr_operand(self, offset - 1) {
            Some(*self)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Z80Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let instruction = dez80::Instruction::decode_one(&mut self.encode().as_slice()).unwrap();
        write!(f, "{}", instruction)
    }
}

/// The Z80 instruction set.
#[derive(Clone, Copy, Debug, Default)]
pub struct Z80InstructionSet {
    i8080: bool,
}

impl Z80InstructionSet {
    /// limits the instruction selection to instructions that are available on the Intel 8080.
    pub fn i8080(&mut self) -> Self {
        self.i8080 = true;
        *self
    }
}

impl InstructionSet for Z80InstructionSet {
    type Instruction = Z80Instruction;

    fn random(&self) -> Self::Instruction {
        todo!()
    }

    fn next(&self, instruction: &mut Self::Instruction) -> Option<()> {
        instruction.increment()?;
        if self.i8080 {
            instruction.i8080_fixup();
        }
        Some(())
    }

    fn mutate(&self, _instruction: &mut Self::Instruction) {
        todo!()
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn disassembly() {
        use crate::InstructionSet;
        for p in crate::z80::z80().bruteforce_with_maximum_length(1) {
            p.disassemble();
        }
    }

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
    fn instruction_set_increment() {
        use crate::Instruction;
        use crate::InstructionSet;
        let mut i = super::Z80Instruction::first();
        let p = super::Z80InstructionSet::default();
        assert!(p.next(&mut i).is_some());
        assert!(p.next(&mut i).is_some());
        assert!(p.next(&mut i).is_some());
        assert!(p.next(&mut i).is_some());
    }

    #[test]
    fn bruteforce_search() {
        use crate::InstructionSet;
        let mut p = crate::z80::z80().bruteforce_with_maximum_length(1);
        p.next().unwrap();
        p.next().unwrap();
        p.next().unwrap();
    }

    #[test]
    fn the_emulator_can_run_the_instructions() {
        use crate::z80::emulators::*;
        use crate::Emulator;
        use crate::InstructionSet;

        for p in crate::z80::z80().bruteforce_with_maximum_length(1) {
            Z80::default().run(0x8000, &p);
        }

        for p in crate::z80::z80().i8080().bruteforce_with_maximum_length(1) {
            Z80::default().run(0x8000, &p);
            I8080::default().run(0x8000, &p);
        }
    }

    #[test]
    fn lengths() {
        use crate::Instruction;

        use super::Z80Instruction;
        let mut insn = Z80Instruction::first();

        loop {
            if insn.increment().is_none() {
                break;
            }
        }
    }
}
