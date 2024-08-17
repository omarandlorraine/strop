//! Module for representing and working with Motorola 6809 machine instructions.

/// Represents a Motorola 6809 machine instruction.
#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Insn([u8; 5]);

impl crate::Iterable for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn step(&mut self) -> bool {
        use crate::Encode;
        if self.0[0..3] == [0xff, 0xff, 0xff] {
            false
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup();
            true
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        let mut encoding = self.0.to_vec();
        encoding.truncate(self.len());
        encoding
    }

    fn len(&self) -> usize {
        let idat = self.instruction_data();
        assert!(idat.action != "unknown");
        self.instruction_data().size
    }
}

impl Insn {
    /// Decodes and parses the instruction, and returns miscellaneous information
    pub fn instruction_data(&self) -> emu6809::isa::Instruction {
        use emu6809::cpu::InstructionDecoder;
        use emu6809::mem::MemBlock;
        use emu6809::mem::MemReader;
        let mut memblock: MemBlock<emu6809::byteorder::LittleEndian> =
            MemBlock::from_data(0, "test_i9n", &self.0, true);
        InstructionDecoder::new_from_reader(&mut MemReader::new(&mut memblock))
            .unwrap()
            .instruction_info
            .clone()
    }

    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
    }

    /// Returns true if the `Insn` encodes a valid 6809 instruction, and false otherwise.
    pub fn valid(&self) -> bool {
        if self.0[0] == 17 && self.0[1] > 188 {
            // There seems to be a bug in emu6809 which panics when decoding instructions beyond
            // this range. But we know it's not a valid instruction; it's outside of the range of
            // valid instructions. So we return false in this special case.
            return false;
        }
        self.instruction_data().action != "unknown"
    }

    fn index_postbyte_fixup(&mut self, offset: usize) {
        if (0x87..=0x8f).contains(&self.0[offset]) {
            self.0[offset] = 0x90
        };
        if (0x97..=0x9f).contains(&self.0[offset]) {
            self.0[offset] = 0x9f
        };
        if (0xa7..=0xaf).contains(&self.0[offset]) {
            self.0[offset] = 0xb0
        };
        if (0xb7..=0xbf).contains(&self.0[offset]) {
            self.0[offset] = 0xc0
        };
        if (0xc7..=0xcf).contains(&self.0[offset]) {
            self.0[offset] = 0xd0
        };
        if (0xd7..=0xdf).contains(&self.0[offset]) {
            self.0[offset] = 0xe0
        };
        if (0xe7..=0xef).contains(&self.0[offset]) {
            self.0[offset] = 0xf0
        };
        if self.0[offset] > 0xf6 {
            self.incr_at_offset(offset - 1);
            self.0[offset] = 0;
        }
    }

    fn tfr_operand_fixup(&mut self, offset: usize) {
        fn a(o: u8) -> bool {
            o & 0x08 == 0
        }
        fn invalid(o: u8) -> bool {
            let o = o & 0x0f;
            [0x06, 0x07, 0x0c, 0x0d, 0x0e, 0x0f].contains(&o)
        }
        while a(self.0[offset]) != a(self.0[offset] >> 4)
            || invalid(self.0[offset])
            || invalid(self.0[offset] >> 4)
        {
            self.incr_at_offset(offset);
        }
    }

    fn opcode_fixup(&mut self, byte: u8) -> bool {
        use std::cmp::Ordering;
        match self.0[0].cmp(&byte) {
            Ordering::Greater => false,
            Ordering::Less => {
                self.0[0] = byte;
                true
            }
            Ordering::Equal => true,
        }
    }

    fn second_opcode_fixup(&mut self, byte: u8) -> bool {
        use std::cmp::Ordering;
        match self.0[1].cmp(&byte) {
            Ordering::Greater => false,
            Ordering::Less => {
                self.0[1] = byte;
                true
            }
            Ordering::Equal => true,
        }
    }

    /// Whatever the values of the bytes, this method mutates them such that they are guaranteed to
    /// represent a valid 6809 instruction.
    pub fn fixup(&mut self) {
        // skip illegal opcodes in the range $00...$0f
        for opcode in [0x00, 0x03, 0x06, 0x0c] {
            if self.opcode_fixup(opcode) {
                return;
            }
        }

        if self.opcode_fixup(0x10) {
            // skip illegal opcodes in the range $1000...$100f
            for opcode in [0x21, 0x3f, 0x83, 0x8c, 0x8e, 0x93, 0x9c, 0x9e] {
                if self.second_opcode_fixup(opcode) {
                    return;
                }
            }

            // skip illegal opcodes in the range $10a0...$10af, but also make sure that the index
            // postbyte is valid
            for opcode in [0xa3, 0xac, 0xae, 0xaf] {
                if self.second_opcode_fixup(opcode) {
                    self.index_postbyte_fixup(2);
                    if self.0[1] == opcode {
                        return;
                    }
                }
            }

            // skip illegal opcodes in the range $10b0...$10df
            for opcode in [0xb3, 0xbc, 0xbe, 0xce, 0xde] {
                if self.second_opcode_fixup(opcode) {
                    return;
                }
            }

            // skip illegal opcodes in the range $10e0...$10ff, but also make sure that the index
            // postbyte is valid
            for opcode in [0xee, 0xef, 0xfe] {
                if self.second_opcode_fixup(opcode) {
                    self.index_postbyte_fixup(2);
                    if self.0[1] == opcode {
                        return;
                    }
                }
            }
            return;
        }

        if self.opcode_fixup(0x11) {
            // skip illegal opcodes in the range $1100...$119f
            for opcode in [0x3f, 0x83, 0x8c, 0x93, 0x9c] {
                if self.second_opcode_fixup(opcode) {
                    return;
                }
            }

            // skip illegal opcodes in the range $11a0...$11af, but also make sure that the index
            // postbyte is valid
            for opcode in [0xa3, 0xac] {
                if self.second_opcode_fixup(opcode) {
                    self.index_postbyte_fixup(2);
                    if self.0[1] == opcode {
                        return;
                    }
                }
            }

            // skip illegal opcodes in the range $11b0...$11ff
            for opcode in [0xb3, 0xbc] {
                if self.second_opcode_fixup(opcode) && self.0[1] == opcode {
                    return;
                }
            }
            self.incr_at_offset(0);
        }

        // skip illegal opcodes in the range $12...$1b
        for opcode in [0x16, 0x19, 0x1c] {
            if self.opcode_fixup(opcode) {
                return;
            }
        }

        // instructions in the range $1e...$1f need to have a valid register operand combo
        for opcode in [0x1e, 0x1f] {
            if self.opcode_fixup(opcode) {
                self.tfr_operand_fixup(1);
                if self.0[0] == opcode {
                    return;
                }
            }
        }

        // instructions in the range $30...$33 need a valid indexed addressing postbyte
        for opcode in 0x30..=0x33 {
            if self.opcode_fixup(opcode) {
                self.index_postbyte_fixup(1);
                if self.0[0] == opcode {
                    return;
                }
            }
        }

        // skip illegal instructions in the range $38...$5f
        for opcode in [0x39, 0x3f, 0x43, 0x46, 0x4c, 0x53, 0x56, 0x5c, 0x5f] {
            if self.opcode_fixup(opcode) {
                return;
            }
        }

        // next up are some opcodes which have an index postbyte, so fix that up
        for opcode in [
            0x60, 0x63, 0x64, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6c, 0x6d, 0x6e, 0x6f, 0x73, 0x76,
            0x7c, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac,
            0xad, 0xae, 0xaf,
        ] {
            if self.opcode_fixup(opcode) {
                self.index_postbyte_fixup(1);
                if self.0[0] == opcode {
                    return;
                }
            }
        }

        // skip illegal instructions in the range $bc...$df
        for opcode in [0xc8, 0xce, 0xd0] {
            if self.opcode_fixup(opcode) {
                return;
            }
        }

        for opcode in 0xe0..=0xef {
            if self.opcode_fixup(opcode) {
                self.index_postbyte_fixup(1);
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_opcodes() {
        use super::Insn;
        use crate::Encode;
        use crate::Iterable;

        let mut i = Insn::first();
        while i.step() {
            let insndat = i.instruction_data();
            println!("{:?}", i);
            let disassembly = format!("{}", i);
            assert!(
                !disassembly.contains("SET TBD"),
                "The disassembly for {} ({:?}) contains the forbidden substring: SET TBD",
                i,
                i
            );
            assert!(
                !disassembly.contains("ILLEGAL"),
                "The disassembly for {} ({:?}) contains the forbidden substring: ILLEGAL",
                i,
                i
            );
            assert_ne!(
                insndat.action, "unknown",
                "{:?} is not a valid instruction, yet it is visited by the .increment() method",
                i
            );
            assert_eq!(
                insndat.size,
                i.encode().len(),
                "{:?} has the wrong length",
                i
            );
        }
    }
}
