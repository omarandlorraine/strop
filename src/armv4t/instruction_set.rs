//!  Two instruction sets supported by the ARMv4T.

use crate::Instruction;

#[derive(Clone, Copy)]
pub struct Thumb(pub u16);
#[derive(Clone, Copy)]
pub struct Arm(pub u32);

impl Instruction for Thumb {
    fn random() -> Self {
        use rand::random;
        Self(random())
    }

    fn mutate(self) -> Self {
        use rand::Rng;
        let bit = rand::thread_rng().gen_range(0..16);
        let encoding = self.0 ^ (1 << bit);

        Self(encoding)
    }

    fn encode(self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    fn first() -> Self {
        Self(0)
    }

    fn increment(&mut self) -> Option<Self> {
        let mut encoding = self.0.checked_add(1)?;

        // Skip the instructions that can access High Registers, but don't.
        // Although the emulator handles them gracefully,  they are explicitly marked as
        // "Unpredictable" by Arm
        if encoding & 0xfcc0 == 0x4400 {
            encoding += 0x0040;
        }

        // From what I understand, the three lowest bits of a BX instruction all have to be zero
        if encoding & 0xff07 == 0x4701 {
            encoding += 0x07;
        }

        if encoding & 0xff80 == 0xb000 {
            encoding |= 0x0080;
        }

        // Skip undecodable instructions in range 0xb100..0xb3ff
        if encoding == 0xb100 {
            encoding = 0xb400;
        }

        // Skip undecodable instructions in range 0xb600..0xb7ff
        if encoding == 0xb600 {
            encoding = 0xbc00;
        }

        // Skip undecodable instructions in range 0xbe00..0xbfff
        if encoding == 0xbe00 {
            encoding = 0xc000;
        }

        // Skip pointless conditional branches where the condition is ALWAYS or NEVER
        if encoding == 0xd600 {
            encoding = 0xd700;
        }

        // Skip undecodable instructions in range 0xe800..0xefff
        if encoding == 0xe800 {
            encoding = 0xf000;
        }
        self.0 = encoding;
        Some(Thumb(encoding))
    }
}

fn unpredictable_instruction(insn: &Thumb) -> Option<Thumb> {
    // If it's an unpredictable instruction, then return the next instruction which isn't an
    // unpredictable instruction. (Next instruction here means, the one who's encoding is next
    // in increasing numerical order)
    if insn.0 & 0xffc0 == 0x4400 {
        // `add rd, rm` encoding, which, because it doesn't use high registers, is redundant
        // with the three-operand `add rd, rn, rm` instruction. Set the high-bit for Rm.
        Some(Thumb(0x4440))
    } else if insn.0 & 0xffc0 == 0x4500 {
        // `cmp rm, rn` encoding, which, because it doesn't use high registers, is redundant
        // with the three-operand `cmp rd, rn, rm` instruction. Set the high-bit for Rn.
        Some(Thumb(0x4540))
    } else if insn.0 & 0xffc0 == 0x4600 {
        // `mov rd, rm` encoding, which doesn't use high registers. This does not have any
        // equivalent instruction, but is still marked as unpredictable. Set the high-bit
        // for Rm.
        Some(Thumb(0x4640))
    } else if insn.0 & 0xff04 == 0x4704 {
        // encodings for `bl rm` and `blx rm`. These need zeroes in the bottom three bits, so
        // some of the non-zero values are marked as unpredictable
        Some(Thumb((insn.0 | 0x0007) + 1))
    } else if insn.0 & 0xff00 == 0xb100 {
        // No instructions in this range; the next one is `push {<nothing>}`
        Some(Thumb(0xb400))
    } else if insn.0 & 0xfa00 == 0xb200 {
        // No instructions in this range; the next one is `pop {<nothing>}`
        Some(Thumb(0xb600))
    } else if insn.0 & 0xfc00 == 0xb800 {
        // No instructions in this range; the next one is `bkpt #0`
        Some(Thumb(0xbe00))
    } else {
        None
    }
}

#[derive(Clone, Default)]
pub struct ThumbInstructionSet {
    unpredictables: bool,
}

impl ThumbInstructionSet {
    pub fn allow_unpredictable_instructions(&mut self) -> &mut Self {
        self.unpredictables = true;
        self
    }
}

impl crate::InstructionSet for ThumbInstructionSet {
    type Instruction = Thumb;

    fn next(&self, thumb: &mut Self::Instruction) -> Option<()> {
        if !self.unpredictables {
            if let Some(new_instruction) = unpredictable_instruction(thumb) {
                *thumb = new_instruction;
            }
        }
        Some(())
    }
}

mod disassembly {
    use super::Thumb;

    impl std::fmt::Display for Thumb {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            // There may be loads of mistakes in here.
            let registers = ["r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7"];
            let high_registers = ["r8", "r9", "r10", "r11", "r12", "sp", "lr", "pc"];
            if self.0 & 0xf800 == 0x1800 {
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[((self.0 & 0x07) >> 3) as usize];
                let imm = self.0 >> 3 & 0x07;

                let op = if self.0 & 0x0200 == 0 { "add" } else { "sub" };

                if self.0 & 0x0400 == 0 {
                    write!(f, "{} {}, {}, {}", op, rd, rn, registers[imm as usize])
                } else {
                    write!(f, "{} {}, {}, #{}", op, rd, rn, imm)
                }
            } else if self.0 & 0xe000 == 0x0000 {
                let opcodes = ["lsl", "lsr", "asr"];

                let rd = registers[(self.0 & 0x07) as usize];
                let rs = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x1f;
                let opcode = opcodes[(self.0 >> 11 & 0x3) as usize];

                write!(f, "{} {}, {}, {}", opcode, rs, rd, offset)
            } else if self.0 & 0xe000 == 0x2000 {
                let opcodes = ["cmp", "mov", "add", "sub"];

                let r = registers[(self.0 >> 8 & 0x07) as usize];
                let imm = self.0 & 0x00ff;
                let opcode = opcodes[(self.0 >> 11 & 0x3) as usize];

                write!(f, "{} {}, #{}", opcode, r, imm)
            } else if self.0 & 0xfc00 == 0x4000 {
                let opcodes = [
                    "and", "eor", "lsl", "lsr", "asr", "adc", "sbc", "ror", "tst", "neg", "cmp",
                    "cmn", "orr", "mul", "bic", "mvn",
                ];
                let rd = registers[(self.0 & 0x07) as usize];
                let rs = registers[(self.0 >> 3 & 0x07) as usize];
                let opcode = opcodes[(self.0 >> 6 & 0x0f) as usize];
                write!(f, "{} {}, {}", opcode, rd, rs)
            } else if self.0 & 0xfc00 == 0x4400 && self.0 & 0x0300 != 0x0300 {
                // These are opcodes add, mov, and cmp, which can access high registers and low
                // registers.

                let opcodes = ["add", "cmp", "mov"];
                let rd = if self.0 & 0x0040 != 0x00 {
                    high_registers[(self.0 >> 3 & 0x07) as usize]
                } else {
                    registers[(self.0 >> 3 & 0x07) as usize]
                };

                let rm = if self.0 & 0x0080 != 0x00 {
                    high_registers[(self.0 & 0x07) as usize]
                } else {
                    registers[(self.0 & 0x07) as usize]
                };
                let opcode = opcodes[(self.0 >> 8 & 0x03) as usize];
                write!(f, "{} {}, {}", opcode, rd, rm)
            } else if self.0 & 0xff07 == 0x4700 {
                let rd = if self.0 & 0x0040 != 0x00 {
                    high_registers[(self.0 >> 3 & 0x07) as usize]
                } else {
                    registers[(self.0 >> 3 & 0x07) as usize]
                };
                write!(f, "bx {}", rd)
            } else if self.0 & 0xf000 == 0x5000 {
                let opcodes = [
                    "str", "strsh", "strb", "strsb", "ldr", "ldrsh", "ldrb", "ldrsb",
                ];
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let rm = registers[(self.0 >> 6 & 0x07) as usize];
                let opcode = opcodes[(self.0 >> 9 & 0x03) as usize];
                write!(f, "{} {}, [{}, {}]", opcode, rd, rn, rm)
            } else if self.0 & 0xf800 == 0x4800 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                write!(f, "ldr {}, [pc, {}]", rd, self.0 & 0xff)
            } else if self.0 & 0xe000 == 0x6000 {
                let opcodes = ["str", "ldr", "strb", "ldrb"];
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x01f;
                let opcode = opcodes[(self.0 >> 11 & 0x03) as usize];

                let scale = if self.0 & 0x0800 == 0 { 4 } else { 1 };

                write!(f, "{} {}, [{}, {}]", opcode, rd, rn, offset * scale)
            } else if self.0 & 0xf000 == 0x8000 {
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x01f;

                if self.0 & 0x1000 == 0 {
                    write!(f, "strh {}, [{}, {}]", rd, rn, offset * 2)
                } else {
                    write!(f, "ldrh {}, [{}, {}]", rd, rn, offset * 2)
                }
            } else if self.0 & 0xf000 == 0x9000 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                let offset = self.0 & 0x00ff;

                if self.0 & 0x0800 == 0 {
                    write!(f, "str {}, [pc, {}]", rd, offset * 4)
                } else {
                    write!(f, "ldr {}, [pc, {}]", rd, offset * 4)
                }
            } else if self.0 & 0xf000 == 0xa000 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                let offset = self.0 & 0x00ff;

                if self.0 & 0x0800 == 0 {
                    write!(f, "add {}, [pc, {}]", rd, offset * 4)
                } else {
                    write!(f, "add {}, [sp, {}]", rd, offset * 4)
                }
            } else if self.0 & 0xff80 == 0xb080 {
                let value = self.0 & 0x007f;
                write!(f, "sub sp, sp, {}", value)
            } else if self.0 & 0xfe00 == 0xb400 {
                let r0 = if self.0 & 0x01 != 0 { "r0, " } else { "" };
                let r1 = if self.0 & 0x02 != 0 { "r1, " } else { "" };
                let r2 = if self.0 & 0x04 != 0 { "r2, " } else { "" };
                let r3 = if self.0 & 0x08 != 0 { "r3, " } else { "" };
                let r4 = if self.0 & 0x10 != 0 { "r4, " } else { "" };
                let r5 = if self.0 & 0x20 != 0 { "r5, " } else { "" };
                let r6 = if self.0 & 0x40 != 0 { "r6, " } else { "" };
                let r7 = if self.0 & 0x40 != 0 { "r7, " } else { "" };
                let lr = if self.0 & 0x100 != 0 { "lr, " } else { "" };
                write!(
                    f,
                    "push {{{}{}{}{}{}{}{}{}{}}}",
                    r0, r1, r2, r3, r4, r5, r6, r7, lr
                )
            } else if self.0 & 0xfe00 == 0xbc00 {
                let r0 = if self.0 & 0x01 != 0 { "r0, " } else { "" };
                let r1 = if self.0 & 0x02 != 0 { "r1, " } else { "" };
                let r2 = if self.0 & 0x04 != 0 { "r2, " } else { "" };
                let r3 = if self.0 & 0x08 != 0 { "r3, " } else { "" };
                let r4 = if self.0 & 0x10 != 0 { "r4, " } else { "" };
                let r5 = if self.0 & 0x20 != 0 { "r5, " } else { "" };
                let r6 = if self.0 & 0x40 != 0 { "r6, " } else { "" };
                let r7 = if self.0 & 0x40 != 0 { "r7, " } else { "" };
                let pc = if self.0 & 0x100 != 0 { "pc, " } else { "" };
                write!(
                    f,
                    "pop {{{}{}{}{}{}{}{}{}{}}}",
                    r0, r1, r2, r3, r4, r5, r6, r7, pc
                )
            } else if self.0 & 0xf000 == 0xc000 {
                let r0 = if self.0 & 0x01 != 0 { "r0, " } else { "" };
                let r1 = if self.0 & 0x02 != 0 { "r1, " } else { "" };
                let r2 = if self.0 & 0x04 != 0 { "r2, " } else { "" };
                let r3 = if self.0 & 0x08 != 0 { "r3, " } else { "" };
                let r4 = if self.0 & 0x10 != 0 { "r4, " } else { "" };
                let r5 = if self.0 & 0x20 != 0 { "r5, " } else { "" };
                let r6 = if self.0 & 0x40 != 0 { "r6, " } else { "" };
                let r7 = if self.0 & 0x40 != 0 { "r7, " } else { "" };
                let rn = registers[(self.0 >> 8 & 0x07) as usize];
                if self.0 & 0x0800 != 0 {
                    write!(
                        f,
                        "stmia {}!, {{{}{}{}{}{}{}{}{}}}",
                        rn, r0, r1, r2, r3, r4, r5, r6, r7
                    )
                } else {
                    write!(
                        f,
                        "ldmia {}!, {{{}{}{}{}{}{}{}{}}}",
                        rn, r0, r1, r2, r3, r4, r5, r6, r7
                    )
                }
            } else if self.0 & 0xf000 == 0xd000 {
                let opcodes = [
                    "eq", "ne", "cs", "cc", "hs", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge",
                    "lt", "gt", "le",
                ];
                let offset = self.0.to_be_bytes()[1] as i8;
                let syscall = self.0 & 0xff;
                let func = (self.0 >> 8 & 0x0f) as usize;

                #[allow(clippy::comparison_chain)]
                if func < 15 {
                    write!(f, "b{} {}", opcodes[func], offset)
                } else if func == 0x0f {
                    write!(f, "swi {}", syscall)
                } else {
                    write!(f, "0x{:04x}", self.0)
                }
            } else if self.0 & 0xf800 == 0xe000 {
                write!(f, "b {}", self.0 & 0x7ff)
            } else if self.0 & 0xf800 == 0xf000 {
                write!(f, "bl {} (+)", self.0 & 0x7ff) // I have no idea what this means
            } else if self.0 & 0xf800 == 0xf800 {
                write!(f, "bl {}", self.0 & 0x7ff)
            } else {
                write!(f, "0x{:04x}", self.0)
            }
        }
    }
}
