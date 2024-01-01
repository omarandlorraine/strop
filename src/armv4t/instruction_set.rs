//!  Two instruction sets supported by the ARMv4T.


use crate::Instruction;

/// Type representing the Thumb instruction (no Thumb2 instructions are present here. It's just the
/// first, fixed-width version).
#[derive(Clone, Copy, Debug)]
pub struct Thumb(pub u16);

/// Type representing the full-width ARM instruction.
#[derive(Clone, Copy, Debug)]
pub struct Arm(pub u32);

impl Instruction for Thumb {
    fn random() -> Self {
        use rand::random;
        let thumb = Self(random());
        if let Some(next) = undefined_instruction(&thumb) {
            next
        } else {
            thumb
        }
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
        self.0 = self.0.checked_add(1)?;
        if let Some(next) = undefined_instruction(self) {
            *self = next;
        }
        Some(*self)
    }
}

fn control_flow(insn: &Thumb) -> Option<Thumb> {
    // If it's a control flow instruction, then return the next instruction which isn't.
    if insn.0 & 0xff00 == 0x4700 {
        Some(Thumb((insn.0 | 0x07ff) + 1))
    } else {
        None
    }
}

fn undefined_instruction(insn: &Thumb) -> Option<Thumb> {
    // If it's an undefined instruction, then return the next instruction which is actually
    // a defined instruction. (Next instruction here means, the one who's encoding is next
    // in increasing numerical order)
    if insn.0 & 0xff04 == 0x4700 {
        // encodings for `bl rm` and `blx rm`. These need zeroes in the bottom three bits, so
        // some of the non-zero values are marked as unpredictable
        Some(Thumb((insn.0 | 0x0007) + 1))
    } else if insn.0 & 0xff00 == 0xb100 {
        // No instructions in this range; the next one is `push {<nothing>}`
        Some(Thumb(0xb400))
    } else if insn.0 & 0xfe00 == 0xb600 {
        // for push instructions, bit 10 (0x0200) needs to be zero.
        Some(Thumb(0xfc00))
    } else {
        None
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
    } else {
        None
    }
}

fn load_store_instruction(insn: &Thumb) -> Option<Thumb> {
    // If it's an load/store instruction, then return the next instruction which isn't a
    // load/store instruction.

    if insn.0 >= 0x4800 && insn.0 < 0xb000 {
        // looks like all load/store instructions are in this range. This range also includes adds
        // relative to SP or PC, which have the same addressing modes. So just return the next
        // instruction which is `sub sp, sp, 0`.
        Some(Thumb(0xb080))
    } else {
        None
    }
}

/// The instruction set known by the ARMv4T in Thumb mode
#[derive(Clone, Default, Debug)]
pub struct ThumbInstructionSet {
    unpredictables: bool,
    branchless: bool,
}

impl ThumbInstructionSet {
    /// An ordinary `ThumbInstructionSet`
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the `ThumbInstructionSet` to consider valid encodings which are so-called
    /// "Unpredictable". I have no idea how the third-party emulator emulates these.
    pub fn allow_unpredictable_instructions(&mut self) -> &mut Self {
        self.unpredictables = true;
        self
    }

    /// Makes sure not to select any branches or other control flow instructions.
    pub fn branchless(&mut self) -> &mut Self {
        self.branchless = true;
        self
    }

    fn check(&self, thumb: &Thumb) -> Option<Thumb> {
        if let Some(new_instruction) = load_store_instruction(thumb) {
            return Some(new_instruction);
        }
        if !self.unpredictables {
            if let Some(new_instruction) = unpredictable_instruction(thumb) {
                return Some(new_instruction);
            }
        }
        if self.branchless {
            if let Some(new_instruction) = control_flow(thumb) {
                return Some(new_instruction);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_thumb_instructions_have_disassembly() {
        use crate::armv4t::instruction_set::Thumb;
        use crate::armv4t::instruction_set::ThumbInstructionSet;
        use crate::Instruction;

        let mut thumb = Thumb::first();
        while thumb.increment().is_some() {
            let dasm = format!("{}", thumb);
            assert!(!dasm.starts_with("0x"), "no disassembly for {}", dasm);
        }
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
                    write!(
                        f,
                        "{} {}, {}, {}   ; {:#06x}",
                        op, rd, rn, registers[imm as usize], self.0
                    )
                } else {
                    write!(f, "{} {}, {}, #{}     ; {:#06x}", op, rd, rn, imm, self.0)
                }
            } else if self.0 & 0xe000 == 0x0000 {
                let opcodes = ["lsl", "lsr", "asr"];

                let rd = registers[(self.0 & 0x07) as usize];
                let rs = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x1f;
                let opcode = opcodes[(self.0 >> 11 & 0x3) as usize];

                write!(
                    f,
                    "{} {}, {}, {}     ; {:#06x}",
                    opcode, rs, rd, offset, self.0
                )
            } else if self.0 & 0xe000 == 0x2000 {
                let opcodes = ["cmp", "mov", "add", "sub"];

                let r = registers[(self.0 >> 8 & 0x07) as usize];
                let imm = self.0 & 0x00ff;
                let opcode = opcodes[(self.0 >> 11 & 0x3) as usize];

                write!(f, "{} {}, #{}     ; {:#06x}", opcode, r, imm, self.0)
            } else if self.0 & 0xfc00 == 0x4000 {
                let opcodes = [
                    "and", "eor", "lsl", "lsr", "asr", "adc", "sbc", "ror", "tst", "neg", "cmp",
                    "cmn", "orr", "mul", "bic", "mvn",
                ];
                let rd = registers[(self.0 & 0x07) as usize];
                let rs = registers[(self.0 >> 3 & 0x07) as usize];
                let opcode = opcodes[(self.0 >> 6 & 0x0f) as usize];
                write!(f, "{} {}, {}     ; {:#06x}", opcode, rd, rs, self.0)
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
                write!(f, "{} {}, {}     ; {:#06x}", opcode, rd, rm, self.0)
            } else if self.0 & 0xff07 == 0x4700 {
                let rd = if self.0 & 0x0040 != 0x00 {
                    high_registers[(self.0 >> 3 & 0x07) as usize]
                } else {
                    registers[(self.0 >> 3 & 0x07) as usize]
                };
                write!(f, "bx {}     ; {:#06x}", rd, self.0)
            } else if self.0 & 0xf000 == 0x5000 {
                let opcodes = [
                    "str", "strsh", "strb", "strsb", "ldr", "ldrsh", "ldrb", "ldrsb",
                ];
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let rm = registers[(self.0 >> 6 & 0x07) as usize];
                let opcode = opcodes[(self.0 >> 9 & 0x03) as usize];
                write!(
                    f,
                    "{} {}, [{}, {}]     ; {:#06x}",
                    opcode, rd, rn, rm, self.0
                )
            } else if self.0 & 0xf800 == 0x4800 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                write!(
                    f,
                    "ldr {}, [pc, {}]     ; {:#06x}",
                    rd,
                    self.0 & 0xff,
                    self.0
                )
            } else if self.0 & 0xe000 == 0x6000 {
                let opcodes = ["str", "ldr", "strb", "ldrb"];
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x01f;
                let opcode = opcodes[(self.0 >> 11 & 0x03) as usize];

                let scale = if self.0 & 0x0800 == 0 { 4 } else { 1 };

                write!(
                    f,
                    "{} {}, [{}, {}]     ; {:#06x}",
                    opcode,
                    rd,
                    rn,
                    offset * scale,
                    self.0
                )
            } else if self.0 & 0xf000 == 0x8000 {
                let rd = registers[(self.0 & 0x07) as usize];
                let rn = registers[(self.0 >> 3 & 0x07) as usize];
                let offset = self.0 >> 6 & 0x01f;

                if self.0 & 0x1000 == 0 {
                    write!(
                        f,
                        "strh {}, [{}, {}]     ; {:#06x}",
                        rd,
                        rn,
                        offset * 2,
                        self.0
                    )
                } else {
                    write!(
                        f,
                        "ldrh {}, [{}, {}]     ; {:#06x}",
                        rd,
                        rn,
                        offset * 2,
                        self.0
                    )
                }
            } else if self.0 & 0xf000 == 0x9000 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                let offset = self.0 & 0x00ff;

                if self.0 & 0x0800 == 0 {
                    write!(f, "str {}, [pc, {}]     ; {:#06x}", rd, offset * 4, self.0)
                } else {
                    write!(f, "ldr {}, [pc, {}]     ; {:#06x}", rd, offset * 4, self.0)
                }
            } else if self.0 & 0xf000 == 0xa000 {
                let rd = registers[(self.0 >> 8 & 0x07) as usize];
                let offset = self.0 & 0x00ff;

                if self.0 & 0x0800 == 0 {
                    write!(f, "add {}, [pc, {}]     ; {:#06x}", rd, offset * 4, self.0)
                } else {
                    write!(f, "add {}, [sp, {}]     ; {:#06x}", rd, offset * 4, self.0)
                }
            } else if self.0 & 0xff80 == 0xb080 {
                let value = self.0 & 0x007f;
                write!(f, "sub sp, sp, {}     ; {:#06x}", value, self.0)
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
                    "push {{{}{}{}{}{}{}{}{}{}}}     ; {:#06x}",
                    r0, r1, r2, r3, r4, r5, r6, r7, lr, self.0
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
                    "pop {{{}{}{}{}{}{}{}{}{}}}     ; {:#06x}",
                    r0, r1, r2, r3, r4, r5, r6, r7, pc, self.0
                )
            } else if self.0 & 0xff00 == 0xb000 {
                let imm = self.0.to_le_bytes()[0] as i8;
                if imm < 0 {
                    write!(f, "sub sp, #{}     ; {:#06x}", 0 - imm, self.0)
                } else {
                    write!(f, "add sp, #{}     ; {:#06x}", imm, self.0)
                }
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
                        "stmia {}!, {{{}{}{}{}{}{}{}{}}}     ; {:#06x}",
                        rn, r0, r1, r2, r3, r4, r5, r6, r7, self.0
                    )
                } else {
                    write!(
                        f,
                        "ldmia {}!, {{{}{}{}{}{}{}{}{}}}     ; {:#06x}",
                        rn, r0, r1, r2, r3, r4, r5, r6, r7, self.0
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
                    write!(f, "b{} {}     ; {:#06x}", opcodes[func], offset, self.0)
                } else if func == 0x0f {
                    write!(f, "swi {}     ; {:#06x}", syscall, self.0)
                } else {
                    write!(f, "0x{:04x}     ; {:#06x}", self.0, self.0)
                }
            } else if self.0 & 0xf800 == 0xe000 {
                write!(f, "b {}     ; {:#06x}", self.0 & 0x7ff, self.0)
            } else if self.0 & 0xf800 == 0xf000 {
                write!(f, "bl {} (+)     ; {:#06x}", self.0 & 0x7ff, self.0) // I have no idea what this means
            } else if self.0 & 0xf800 == 0xf800 {
                write!(f, "bl {}     ; {:#06x}", self.0 & 0x7ff, self.0)
            } else {
                write!(f, "; {:#06x}", self.0)
            }
        }
    }
}
