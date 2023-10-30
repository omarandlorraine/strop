//! A module representing the MOS 6502's instruction set in a way that facilitates its use by
//! strop.

use crate::Candidate;
use crate::Instruction;
use crate::InstructionSet;
use rand::random;

type Encoding6502 = [u8; 3];

const NMOS_OPCODES: [u8; 216] = [
    0x00, 0x01, 0x03, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x13,
    0x15, 0x16, 0x17, 0x18, 0x19, 0x1b, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x23, 0x24, 0x25, 0x26, 0x27,
    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x33, 0x35, 0x36, 0x37, 0x38, 0x39,
    0x3b, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x43, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d,
    0x4e, 0x4f, 0x50, 0x51, 0x53, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5b, 0x5d, 0x5e, 0x5f, 0x60, 0x61,
    0x63, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x73, 0x75,
    0x76, 0x77, 0x78, 0x79, 0x7b, 0x7d, 0x7e, 0x7f, 0x81, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x8a,
    0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b,
    0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab,
    0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc,
    0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd,
    0xce, 0xcf, 0xd0, 0xd1, 0xd3, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xdb, 0xdd, 0xde, 0xdf, 0xe0, 0xe1,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf3, 0xf5,
    0xf6, 0xf7, 0xf8, 0xf9, 0xfb, 0xfd, 0xfe, 0xff,
];

fn next_nmos_opcode(opcode: u8) -> Option<u8> {
    let index = NMOS_OPCODES.iter().position(|&r| r == opcode)? + 1;
    if index >= NMOS_OPCODES.len() {
        return None;
    }

    Some(NMOS_OPCODES[index])
}

const CMOS_OPCODES: [u8; 178] = [
    0x00, 0x01, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0a, 0x0c, 0x0d, 0x0e, 0x10, 0x11, 0x12, 0x14, 0x15,
    0x16, 0x18, 0x19, 0x1a, 0x1c, 0x1d, 0x1e, 0x20, 0x21, 0x24, 0x25, 0x26, 0x28, 0x29, 0x2a, 0x2c,
    0x2d, 0x2e, 0x30, 0x31, 0x32, 0x34, 0x35, 0x36, 0x38, 0x39, 0x3a, 0x3c, 0x3d, 0x3e, 0x40, 0x41,
    0x45, 0x46, 0x48, 0x49, 0x4a, 0x4c, 0x4d, 0x4e, 0x50, 0x51, 0x52, 0x55, 0x56, 0x58, 0x59, 0x5a,
    0x5d, 0x5e, 0x60, 0x61, 0x64, 0x65, 0x66, 0x68, 0x69, 0x6a, 0x6c, 0x6d, 0x6e, 0x70, 0x71, 0x72,
    0x74, 0x75, 0x76, 0x78, 0x79, 0x7a, 0x7c, 0x7d, 0x7e, 0x80, 0x81, 0x84, 0x85, 0x86, 0x88, 0x89,
    0x8a, 0x8c, 0x8d, 0x8e, 0x90, 0x91, 0x92, 0x94, 0x95, 0x96, 0x98, 0x99, 0x9a, 0x9c, 0x9d, 0x9e,
    0xa0, 0xa1, 0xa2, 0xa4, 0xa5, 0xa6, 0xa8, 0xa9, 0xaa, 0xac, 0xad, 0xae, 0xb0, 0xb1, 0xb2, 0xb4,
    0xb5, 0xb6, 0xb8, 0xb9, 0xba, 0xbc, 0xbd, 0xbe, 0xc0, 0xc1, 0xc4, 0xc5, 0xc6, 0xc8, 0xc9, 0xca,
    0xcc, 0xcd, 0xce, 0xd0, 0xd1, 0xd2, 0xd5, 0xd6, 0xd8, 0xd9, 0xda, 0xdd, 0xde, 0xe0, 0xe1, 0xe4,
    0xe5, 0xe6, 0xe8, 0xe9, 0xea, 0xec, 0xed, 0xee, 0xf0, 0xf1, 0xf2, 0xf5, 0xf6, 0xf8, 0xf9, 0xfa,
    0xfd, 0xfe,
];

/// A struct representing one MOS 6502 instruction
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nmos6502Instruction {
    encoding: Encoding6502,
}

/// A struct representing one MOS 6502 instruction
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cmos6502Instruction {
    encoding: Encoding6502,
}

#[derive(Clone, Debug)]
enum Mos6502StaticAnalysisTypes {
    ReadOnly(Vec<u16>),
    Writeable(Vec<u16>),
    Readable(Vec<u16>),
    NoConditionalBranches,
    NoIllegalInstructions,
    NoIndirectJumpBug,
    SensibleBranchTargets,
    NoRor,
}

/// The instruction set known by NMOS 6502s, including the ROR instruction.
#[derive(Clone, Debug, Default)]
pub struct Mos6502 {
    cmos_compatible: bool,
    reva_compatible: bool,

    basic_block: bool,

    writable_addresses: Vec<u16>,
    readable_addresses: Vec<u16>,

    sa: Vec<Mos6502StaticAnalysisTypes>,
}

/// The instruction set known by CMOS 6502s. This includes such instructions as `phx`, `ply`,
/// `stz`, etc.
#[derive(Clone, Debug, Default)]
pub struct Cmos6502 {
    sa: Vec<Mos6502StaticAnalysisTypes>,
}

impl Mos6502 {
    /// Configures the `InstructionSet` to include only those instructions that are compatible with
    /// the CMOS models of the 6502
    pub fn cmos_compatible(&mut self) -> &mut Self {
        self.cmos_compatible = true;
        self
    }

    /// configures the `InstructionSet` to exclude the `ROR` instruction, which is not present on
    /// very early chips.
    pub fn reva_compatible(&mut self) -> &mut Self {
        self.reva_compatible = true;
        self
    }

    /// Makes sure that any branches actually target instructions (and not, for example, half-way
    /// through a multi-byte instruction, or somewhere before the program's start, or ...)
    pub fn branch_target(&mut self) -> &mut Self {
        self.sa
            .push(Mos6502StaticAnalysisTypes::SensibleBranchTargets);
        self
    }

    /// Makes sure not to include instructions exhibiting the hardware bug that can happen with indirect
    /// jumps through pointers which straddle memory pages.
    pub fn no_indirect_jump_bug(&mut self) -> &mut Self {
        self.sa.push(Mos6502StaticAnalysisTypes::NoIndirectJumpBug);
        self
    }

    /// Specifies writeable addresses. When set, the instructions selected will not write to any
    /// other variables
    pub fn writeable(&mut self, address: u16) -> &mut Self {
        self.writable_addresses.push(address);
        self
    }

    /// Specifies readable addresses. When set, the instructions selected will not read from any
    /// other variables
    pub fn readable(&mut self, address: u16) -> &mut Self {
        self.readable_addresses.push(address);
        self
    }

    /// Excludes any control flow instructions from consideration (e.g. `rts`, `rti`, `jmp`, `beq`,
    /// etc.)
    pub fn basic_block(&mut self) -> &mut Self {
        self.basic_block = true;
        self
    }

    fn check_opcode(&self, insn: &mut Nmos6502Instruction) -> bool {
        #[allow(clippy::if_same_then_else)]
        #[allow(clippy::needless_bool)]
        if self.cmos_compatible && !insn.cmos_compatible() {
            false
        } else if self.reva_compatible && !insn.reva_compatible() {
            false
        } else if self.basic_block && insn.is_control_flow() {
            false
        } else if self.readable_addresses.is_empty() && insn.reads_from().is_some() {
            false
        } else if self.writable_addresses.is_empty() && insn.writes_to().is_some() {
            false
        } else {
            true
        }
    }

    fn address_fixup(&self, insn: &mut Nmos6502Instruction) {
        if let Some(address) = insn.reads_from() {
            if !self.readable_addresses.iter().any(|addr| addr == &address) {
                todo!();
            }
        }
        if let Some(address) = insn.writes_to() {
            if !self.writable_addresses.iter().any(|addr| addr == &address) {
                todo!();
            }
        }
    }
}

impl InstructionSet for Mos6502 {
    type Instruction = Nmos6502Instruction;

    fn next(&self, insn: &mut Self::Instruction) -> Option<()> {
        insn.increment();
        while !self.check_opcode(insn) {
            // If this in an opcode that's not under consideration due to static analysis, then
            // skip this opcode.
            insn.encoding[0] = next_nmos_opcode(insn.encoding[0])?;
        }

        self.address_fixup(insn);

        Some(())
    }

    fn filter(&self, _candidate: &Candidate<Nmos6502Instruction>) -> bool {
        true
    }
}

impl Nmos6502Instruction {
    /// Returns the length of the instruction in bytes. And a 6502 instruction is always either 1,
    /// 2 or 3 bytes.
    pub fn length(&self) -> usize {
        match self.encoding[0] {
            0x00 => 1, // TODO: is this right?
            0x01 => 2,
            0x03 => 2,
            0x05 => 2,
            0x06 => 2,
            0x08 => 1,
            0x07 => 2,
            0x0a => 1,
            0x09 => 2,
            0x0b => 2,
            0x0d => 3,
            0x0e => 3,
            0x0f => 3,
            0x10 => 2,
            0x11 => 2,
            0x13 => 2,
            0x15 => 2,
            0x16 => 2,
            0x17 => 2,
            0x18 => 1,
            0x19 => 3,
            0x1b => 3,
            0x1d => 3,
            0x1e => 3,
            0x1f => 3,
            0x20 => 3,
            0x21 => 2,
            0x23 => 2,
            0x24 => 2,
            0x25 => 2,
            0x26 => 2,
            0x27 => 2,
            0x28 => 1,
            0x29 => 2,
            0x2a => 1,
            0x2b => 2,
            0x2c => 3,
            0x2d => 3,
            0x2e => 3,
            0x2f => 3,
            0x30 => 2,
            0x31 => 2,
            0x33 => 2,
            0x35 => 2,
            0x36 => 2,
            0x37 => 2,
            0x38 => 1,
            0x39 => 3,
            0x3b => 3,
            0x3d => 3,
            0x3e => 3,
            0x3f => 3,
            0x40 => 1,
            0x41 => 2,
            0x43 => 2,
            0x45 => 2,
            0x46 => 2,
            0x47 => 2,
            0x48 => 1,
            0x49 => 2,
            0x4a => 1,
            0x4b => 2,
            0x4c => 3,
            0x4d => 3,
            0x4e => 3,
            0x4f => 3,
            0x50 => 2,
            0x51 => 2,
            0x53 => 2,
            0x55 => 2,
            0x56 => 2,
            0x57 => 2,
            0x58 => 1,
            0x59 => 3,
            0x5b => 3,
            0x5d => 3,
            0x5e => 3,
            0x5f => 3,
            0x60 => 1,
            0x61 => 2,
            0x63 => 2,
            0x65 => 2,
            0x66 => 2,
            0x67 => 2,
            0x68 => 1,
            0x69 => 2,
            0x6a => 1,
            0x6b => 2,
            0x6c => 3,
            0x6d => 3,
            0x6e => 3,
            0x6f => 3,
            0x70 => 2,
            0x71 => 2,
            0x73 => 2,
            0x75 => 2,
            0x76 => 2,
            0x77 => 2,
            0x78 => 1,
            0x79 => 3,
            0x7b => 3,
            0x7d => 3,
            0x7e => 3,
            0x7f => 3,
            0x81 => 2,
            0x83 => 2,
            0x84 => 2,
            0x85 => 2,
            0x86 => 2,
            0x87 => 2,
            0x88 => 1,
            0x8a => 1,
            0x8b => 2,
            0x8c => 3,
            0x8d => 3,
            0x8e => 3,
            0x8f => 3,
            0x90 => 2,
            0x91 => 2,
            0x93 => 2,
            0x94 => 2,
            0x95 => 2,
            0x96 => 2,
            0x97 => 2,
            0x98 => 1,
            0x99 => 3,
            0x9a => 1,
            0x9b => 3,
            0x9c => 3,
            0x9d => 3,
            0x9e => 3,
            0x9f => 3,
            0xa0 => 2,
            0xa1 => 2,
            0xa2 => 2,
            0xa3 => 2,
            0xa4 => 2,
            0xa5 => 2,
            0xa6 => 2,
            0xa7 => 2,
            0xa8 => 1,
            0xa9 => 2,
            0xaa => 1,
            0xab => 2,
            0xac => 3,
            0xad => 3,
            0xae => 3,
            0xaf => 3,
            0xb0 => 2,
            0xb1 => 2,
            0xb3 => 2,
            0xb4 => 2,
            0xb5 => 2,
            0xb6 => 2,
            0xb7 => 2,
            0xb8 => 1,
            0xb9 => 3,
            0xba => 1,
            0xbb => 3,
            0xbc => 3,
            0xbd => 3,
            0xbe => 3,
            0xbf => 3,
            0xc0 => 2,
            0xc1 => 2,
            0xc3 => 2,
            0xc4 => 2,
            0xc5 => 2,
            0xc6 => 2,
            0xc7 => 2,
            0xc8 => 1,
            0xc9 => 2,
            0xca => 1,
            0xcb => 2,
            0xcc => 3,
            0xcd => 3,
            0xce => 3,
            0xcf => 3,
            0xd0 => 2,
            0xd1 => 2,
            0xd3 => 2,
            0xd5 => 2,
            0xd6 => 2,
            0xd7 => 2,
            0xd8 => 1,
            0xd9 => 3,
            0xdb => 3,
            0xdd => 3,
            0xde => 3,
            0xdf => 3,
            0xe0 => 2,
            0xe1 => 2,
            0xe3 => 2,
            0xe4 => 2,
            0xe5 => 2,
            0xe6 => 2,
            0xe7 => 2,
            0xe8 => 1,
            0xe9 => 2,
            0xea => 1,
            0xec => 3,
            0xed => 3,
            0xee => 3,
            0xef => 3,
            0xf0 => 2,
            0xf1 => 2,
            0xf3 => 2,
            0xf5 => 2,
            0xf6 => 2,
            0xf7 => 2,
            0xf8 => 1,
            0xf9 => 3,
            0xfb => 3,
            0xfd => 3,
            0xfe => 3,
            0xff => 3,
            _ => 0,
        }
    }

    /// Returns a new Nmos6502Instruction, from the encoding
    pub fn new(encoding: [u8; 3]) -> Self {
        Self { encoding }
    }

    fn reva_compatible(&self) -> bool {
        !matches!(self.encoding[0], 0x66 | 0x6a | 0x6e | 0x76 | 0x7e)
    }

    fn cmos_compatible(&self) -> bool {
        CMOS_OPCODES.contains(&self.encoding[0]) && !matches!(self.encoding[0], 0x9c | 0x9e)
    }

    fn is_relative_branch(&self) -> bool {
        matches!(
            self.encoding[0],
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0
        )
    }

    fn jmp_indirect_bug(&self) -> bool {
        (self.encoding[0], self.encoding[1]) == (0x6c, 0xff)
    }

    fn is_control_flow(&self) -> bool {
        matches!(
            self.encoding[0],
            0x00 | 0x10
                | 0x20
                | 0x30
                | 0x40
                | 0x4c
                | 0x50
                | 0x60
                | 0x6c
                | 0x70
                | 0x90
                | 0xB0
                | 0xD0
                | 0xF0
        )
    }

    /// If the instruction reads from a memory location, then return that memory location
    fn reads_from(&self) -> Option<u16> {
        #[allow(clippy::if_same_then_else)]
        if self.length() == 1 {
            // The instruction is one byte long and therefore cannot access memory
            None
        } else if self.is_control_flow() {
            // jumps, returns, branches etc. are alos not what we're looking for
            None
        } else if matches!(
            self.encoding[0],
            0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91
        ) {
            // STA instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0x86 | 0x96 | 0x8e) {
            // STX instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0x84 | 0x94 | 0x8c) {
            // STY instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0x87 | 0x97 | 0x8f | 0x83) {
            // SAX instruction is not what we're looking for
            None
        } else if matches!(
            self.encoding[0],
            0x69 | 0x29 | 0xc9 | 0xe0 | 0xc0 | 0x49 | 0xa9 | 0xa2 | 0xa0 | 0x09 | 0xe9
        ) {
            // immediate addressing mode is not what we're looking for
            // TODO: Make sure I haven't forgotten any immediate-mode illegal instructions.
            None
        } else if self.length() == 1 {
            // It's an instruction that reads from zero-page
            Some(self.encoding[1].into())
        } else {
            // It's an instruction that reads from memory somewhere
            Some(u16::from_le_bytes([self.encoding[1], self.encoding[2]]))
        }
    }

    /// If the instruction writes to a memory location, then return that memory location
    fn writes_to(&self) -> Option<u16> {
        #[allow(clippy::if_same_then_else)]
        if self.length() == 1 {
            // The instruction is one byte long and therefore cannot access memory
            None
        } else if self.is_control_flow() {
            // jumps, returns, branches etc. are alos not what we're looking for
            None
        } else if matches!(
            self.encoding[0],
            0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1
        ) {
            // LDA instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe) {
            // LDX instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc) {
            // LDY instruction is not what we're looking for
            None
        } else if matches!(self.encoding[0], 0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3) {
            // LAX instruction is not what w0x84 | 0x94 | 0x8ce're looking for
            None
        } else if self.length() == 1 {
            // It's an instruction that reads from zero-page
            Some(self.encoding[1].into())
        } else {
            // It's an instruction that reads from memory somewhere
            Some(u16::from_le_bytes([self.encoding[1], self.encoding[2]]))
        }
    }
}

impl std::convert::TryFrom<&[u8]> for Nmos6502Instruction {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let temp = Self {
            encoding: [value[0], 0, 0],
        };
        match temp.length() {
            1 => Ok(Self {
                encoding: [value[0], 0, 0],
            }),
            2 => Ok(Self {
                encoding: [value[0], value[1], 0],
            }),
            3 => Ok(Self {
                encoding: [value[0], value[1], value[2]],
            }),
            _ => Err(()),
        }
    }
}

impl std::convert::TryFrom<&[u8]> for Cmos6502Instruction {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let temp = Self {
            encoding: [value[0], 0, 0],
        };
        match temp.length() {
            1 => Ok(Self {
                encoding: [value[0], 0, 0],
            }),
            2 => Ok(Self {
                encoding: [value[0], value[1], 0],
            }),
            3 => Ok(Self {
                encoding: [value[0], value[1], value[2]],
            }),
            _ => Err(()),
        }
    }
}

/// Static analysis pass for excluding anything that's disallowed in a basic block
#[derive(Debug, Default)]
pub struct BasicBlock;

/// Static analysis pass for excluding "illegal opcodes"
#[derive(Debug, Default)]
pub struct ExcludeIllegalInstructions;

/// Static analysis pass for excluding conditional branches that target the middle of some other
/// instruction. For example, `beq -1` ends up executing `$ff`, which is the displacement byte
/// itself, not normally considered a valid opcode. That kind of shenanigan is excluded by this
/// static analysis pass.
#[derive(Debug, Default)]
pub struct BranchTarget;

impl BranchTarget {
    fn permissible_targets(
        cand: &Candidate<Nmos6502Instruction>,
    ) -> Vec<(u16, Nmos6502Instruction)> {
        let mut instructions: Vec<(u16, Nmos6502Instruction)> = vec![];
        let mut offs: u16 = 0;
        for insn in &cand.instructions {
            offs += insn.length() as u16;
            instructions.push((offs, *insn));
        }
        instructions
    }
}

/// A static analysis pass for checking that any memory accesses only happen to/from the correct
/// memory locations. This might be good for, for example, generating a subroutine which reads its
/// parameters from these locations and writes its results to those memory locations.
#[derive(Clone, Debug, Default)]
pub struct VariablesInMemory {
    /// All addresses the instructions may read from
    pub reads: Vec<u16>,
    /// All addressess the instructions may write to
    pub writes: Vec<u16>,
}

impl From<&Candidate<Nmos6502Instruction>> for VariablesInMemory {
    fn from(other: &Candidate<Nmos6502Instruction>) -> Self {
        use std::collections::HashSet;
        // I'm collecting these to a hashset and then iter/collecting to a Vec to deduplicate the
        // values
        other.disassemble();
        let reads: Vec<u16> = other
            .instructions
            .clone()
            .into_iter()
            .filter_map(|insn| insn.reads_from())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let writes: Vec<u16> = other
            .instructions
            .clone()
            .into_iter()
            .filter_map(|insn| insn.writes_to())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        Self { reads, writes }
    }
}

/// The first 6502s have a hardware bug which means a pointer cannot cross a 256-byte page boundary.
/// The one instruction that exercises this bug is the JMP instruction with the indirect addressing
/// mode. This static analysis pass excludes such instructions from consideration.
#[derive(Debug, Default)]
pub struct IndirectJumpBug;

impl Cmos6502Instruction {
    /// Returns the length of the instruction in bytes. And a 6502 instruction is always either 1,
    /// 2 or 3 bytes.
    pub fn length(&self) -> usize {
        match self.encoding[0] {
            0x64 => 2,
            0x74 => 2,
            0xda => 1,
            0x5a => 1,
            0xfa => 1,
            0x7a => 1,
            0x72 => 2,
            0x32 => 2,
            0xd2 => 2,
            0x52 => 2,
            0xb2 => 2,
            0x12 => 2,
            0xf2 => 2,
            0x92 => 2,
            0x04 => 2,
            0x0c => 3,
            0x14 => 2,
            0x1c => 3,
            0x1a => 1,
            0x34 => 2,
            0x3c => 3,
            0x3a => 1,
            0x7c => 3,
            0x89 => 2,
            0x80 => 2,
            0x00 => 1, // TODO: is this right?
            0x01 => 2,
            0x03 => 2,
            0x05 => 2,
            0x06 => 2,
            0x08 => 1,
            0x07 => 2,
            0x0a => 1,
            0x09 => 2,
            0x0b => 2,
            0x0d => 3,
            0x0e => 3,
            0x0f => 3,
            0x10 => 2,
            0x11 => 2,
            0x13 => 2,
            0x15 => 2,
            0x16 => 2,
            0x17 => 2,
            0x18 => 1,
            0x19 => 3,
            0x1b => 3,
            0x1d => 3,
            0x1e => 3,
            0x1f => 3,
            0x20 => 3,
            0x21 => 2,
            0x23 => 2,
            0x24 => 2,
            0x25 => 2,
            0x26 => 2,
            0x27 => 2,
            0x28 => 1,
            0x29 => 2,
            0x2a => 1,
            0x2b => 2,
            0x2c => 3,
            0x2d => 3,
            0x2e => 3,
            0x2f => 3,
            0x30 => 2,
            0x31 => 2,
            0x33 => 2,
            0x35 => 2,
            0x36 => 2,
            0x37 => 2,
            0x38 => 1,
            0x39 => 3,
            0x3b => 3,
            0x3d => 3,
            0x3e => 3,
            0x3f => 3,
            0x40 => 1,
            0x41 => 2,
            0x43 => 2,
            0x45 => 2,
            0x46 => 2,
            0x47 => 2,
            0x48 => 1,
            0x49 => 2,
            0x4a => 1,
            0x4b => 2,
            0x4c => 3,
            0x4d => 3,
            0x4e => 3,
            0x4f => 3,
            0x50 => 2,
            0x51 => 2,
            0x53 => 2,
            0x55 => 2,
            0x56 => 2,
            0x57 => 2,
            0x58 => 1,
            0x59 => 3,
            0x5b => 3,
            0x5d => 3,
            0x5e => 3,
            0x5f => 3,
            0x60 => 1,
            0x61 => 2,
            0x63 => 2,
            0x65 => 2,
            0x66 => 2,
            0x67 => 2,
            0x68 => 1,
            0x69 => 2,
            0x6a => 1,
            0x6b => 2,
            0x6c => 3,
            0x6d => 3,
            0x6e => 3,
            0x6f => 3,
            0x70 => 2,
            0x71 => 2,
            0x73 => 2,
            0x75 => 2,
            0x76 => 2,
            0x77 => 2,
            0x78 => 1,
            0x79 => 3,
            0x7b => 3,
            0x7d => 3,
            0x7e => 3,
            0x7f => 3,
            0x81 => 2,
            0x83 => 2,
            0x84 => 2,
            0x85 => 2,
            0x86 => 2,
            0x87 => 2,
            0x88 => 1,
            0x8a => 1,
            0x8b => 2,
            0x8c => 3,
            0x8d => 3,
            0x8e => 3,
            0x8f => 3,
            0x90 => 2,
            0x91 => 2,
            0x93 => 2,
            0x94 => 2,
            0x95 => 2,
            0x96 => 2,
            0x97 => 2,
            0x98 => 1,
            0x99 => 3,
            0x9a => 1,
            0x9b => 3,
            0x9c => 3,
            0x9d => 3,
            0x9e => 3,
            0x9f => 3,
            0xa0 => 2,
            0xa1 => 2,
            0xa2 => 2,
            0xa3 => 2,
            0xa4 => 2,
            0xa5 => 2,
            0xa6 => 2,
            0xa7 => 2,
            0xa8 => 1,
            0xa9 => 2,
            0xaa => 1,
            0xab => 2,
            0xac => 3,
            0xad => 3,
            0xae => 3,
            0xaf => 3,
            0xb0 => 2,
            0xb1 => 2,
            0xb3 => 2,
            0xb4 => 2,
            0xb5 => 2,
            0xb6 => 2,
            0xb7 => 2,
            0xb8 => 1,
            0xb9 => 3,
            0xba => 1,
            0xbb => 3,
            0xbc => 3,
            0xbd => 3,
            0xbe => 3,
            0xbf => 3,
            0xc0 => 2,
            0xc1 => 2,
            0xc3 => 2,
            0xc4 => 2,
            0xc5 => 2,
            0xc6 => 2,
            0xc7 => 2,
            0xc8 => 1,
            0xc9 => 2,
            0xca => 1,
            0xcb => 2,
            0xcc => 3,
            0xcd => 3,
            0xce => 3,
            0xcf => 3,
            0xd0 => 2,
            0xd1 => 2,
            0xd3 => 2,
            0xd5 => 2,
            0xd6 => 2,
            0xd7 => 2,
            0xd8 => 1,
            0xd9 => 3,
            0xdb => 3,
            0xdd => 3,
            0xde => 3,
            0xdf => 3,
            0xe0 => 2,
            0xe1 => 2,
            0xe3 => 2,
            0xe4 => 2,
            0xe5 => 2,
            0xe6 => 2,
            0xe7 => 2,
            0xe8 => 1,
            0xe9 => 2,
            0xea => 1,
            0xec => 3,
            0xed => 3,
            0xee => 3,
            0xef => 3,
            0xf0 => 2,
            0xf1 => 2,
            0xf3 => 2,
            0xf5 => 2,
            0xf6 => 2,
            0xf7 => 2,
            0xf8 => 1,
            0xf9 => 3,
            0xfb => 3,
            0xfd => 3,
            0xfe => 3,
            0xff => 3,
            _ => 0,
        }
    }

    /// Returns a new Nmos6502Instruction, from the encoding
    pub fn new(encoding: [u8; 3]) -> Self {
        Self { encoding }
    }
}

impl Instruction for Nmos6502Instruction {
    fn random() -> Self {
        use rand::seq::SliceRandom;
        let encoding: [u8; 3] = [
            *NMOS_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            random(),
            random(),
        ];
        Self { encoding }
    }

    fn encode(self) -> Vec<u8> {
        match self.length() {
            1 => vec![self.encoding[0]],
            2 => vec![self.encoding[0], self.encoding[1]],
            3 => vec![self.encoding[0], self.encoding[1], self.encoding[2]],
            _ => panic!(),
        }
    }

    fn mutate(self) -> Self {
        todo!();
    }

    fn first() -> Self {
        Self {
            encoding: [0, 0, 0],
        }
    }

    fn increment(&mut self) -> Option<Self> {
        let length = self.length();

        fn next_opcode(insn: &mut Nmos6502Instruction) -> Option<Nmos6502Instruction> {
            insn.encoding[0] = next_nmos_opcode(insn.encoding[0])?;
            Some(Nmos6502Instruction::new(insn.encoding))
        }

        fn next_lobyte(insn: &mut Nmos6502Instruction) -> Option<Nmos6502Instruction> {
            insn.encoding[1] = insn.encoding[1].wrapping_add(1); // ready for next call
            if insn.encoding[1] == 0 {
                next_opcode(insn)
            } else {
                Some(Nmos6502Instruction::new(insn.encoding))
            }
        }

        fn next_hibyte(insn: &mut Nmos6502Instruction) -> Option<Nmos6502Instruction> {
            insn.encoding[2] = insn.encoding[2].wrapping_add(1); // ready for next call
            if insn.encoding[2] == 0 {
                next_lobyte(insn)
            } else {
                Some(Nmos6502Instruction::new(insn.encoding))
            }
        }

        match length {
            1 => next_opcode(self),
            2 => next_lobyte(self),
            3 => next_hibyte(self),
            _ => panic!(
                "Opcode {}, whose opcode is ${:02x}, has length {}",
                self, self.encoding[0], length
            ),
        }
    }
}

impl Instruction for Cmos6502Instruction {
    fn random() -> Self {
        use rand::seq::SliceRandom;
        let encoding: [u8; 3] = [
            *CMOS_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            random(),
            random(),
        ];
        Self { encoding }
    }

    fn encode(self) -> Vec<u8> {
        todo!();
    }

    fn mutate(self) -> Self {
        todo!();
    }

    fn first() -> Self {
        Self {
            encoding: [0, 0, 0],
        }
    }

    fn increment(&mut self) -> Option<Self> {
        let length = self.length();

        fn next_opcode(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            let index = CMOS_OPCODES.iter().position(|&r| r == insn.encoding[0])? + 1;
            if index > CMOS_OPCODES.len() {
                return None;
            }

            insn.encoding[0] = CMOS_OPCODES[index];
            Some(Cmos6502Instruction::new(insn.encoding))
        }

        fn next_lobyte(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            insn.encoding[1] = insn.encoding[1].wrapping_add(1); // ready for next call
            if insn.encoding[1] == 0 {
                next_opcode(insn)
            } else {
                Some(Cmos6502Instruction::new(insn.encoding))
            }
        }

        fn next_hibyte(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            insn.encoding[2] = insn.encoding[2].wrapping_add(1); // ready for next call
            if insn.encoding[2] == 0 {
                next_lobyte(insn)
            } else {
                Some(Cmos6502Instruction::new(insn.encoding))
            }
        }

        match length {
            1 => next_opcode(self),
            2 => next_lobyte(self),
            3 => next_hibyte(self),
            _ => panic!(
                "Opcode {}, whose opcode is ${:02x}, has length {}",
                self, self.encoding[0], length
            ),
        }
    }
}

mod disassembly {
    use super::Cmos6502Instruction;
    use super::Nmos6502Instruction;

    fn implied(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{}                 ; ${:02x}", opcode, encoding[0])
    }

    fn acc(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{} a               ; ${:02x}", opcode, encoding[0])
    }

    fn indx(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} (${:02x},x)         ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn indy(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} (${:02x}),y         ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn zp(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} ${:02x}             ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn zpi(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} (${:02x})           ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn zpx(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} ${:02x}, x          ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn imm(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} #${:02x}            ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], operand
        )
    }

    fn relative(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        let offset = operand as i8;
        write!(
            f,
            "{} {:<8}        ; ${:02x} ${:02x}",
            opcode, offset, encoding[0], operand
        )
    }

    fn ind(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        write!(
            f,
            "{} (${:02x}{:02x})         ; ${:02x} ${:02x} ${:02x}",
            opcode, oph, opl, encoding[0], encoding[1], encoding[2]
        )
    }

    fn abs(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        write!(
            f,
            "{} ${:02x}{:02x}           ; ${:02x} ${:02x} ${:02x}",
            opcode, oph, opl, encoding[0], encoding[1], encoding[2]
        )
    }

    fn absx(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        write!(
            f,
            "{} ${:02x}{:02x}, x        ; ${:02x} ${:02x} ${:02x}",
            opcode, oph, opl, encoding[0], encoding[1], encoding[2]
        )
    }

    fn absix(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        write!(
            f,
            "{} (${:02x}{:02x}, x)      ; ${:02x} ${:02x} ${:02x}",
            opcode, oph, opl, encoding[0], encoding[1], encoding[2]
        )
    }

    fn absy(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        write!(
            f,
            "{} ${:02x}{:02x}, y        ; ${:02x} ${:02x} ${:02x}",
            opcode, oph, opl, encoding[0], encoding[1], encoding[2]
        )
    }

    fn zpy(
        f: &mut std::fmt::Formatter<'_>,
        opcode: &str,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let operand = encoding[1];
        write!(
            f,
            "{} ${:02x}, y          ; ${:02x} ${:02x}",
            opcode, operand, encoding[0], encoding[1]
        )
    }

    fn bitfields(opcode: u8) -> (u8, u8, u8) {
        // The 6502 opcodes may be seen as having these bitfields: aaabbbcc.
        // All decoding first examines the three fields.
        ((opcode >> 5) & 0x07, (opcode >> 2) & 0x07, opcode & 0x03)
    }

    fn fallback(f: &mut std::fmt::Formatter<'_>, encoding: [u8; 3]) -> Result<(), std::fmt::Error> {
        let oph = encoding[2];
        let opl = encoding[1];
        let opcode = encoding[0];
        write!(
            f,
            ".db {:#04x}, {:#04x}, {:#04x}  ; ({:?})",
            opcode,
            opl,
            oph,
            bitfields(opcode)
        )
    }

    fn common_disassembly(
        f: &mut std::fmt::Formatter<'_>,
        encoding: [u8; 3],
    ) -> Result<(), std::fmt::Error> {
        let opcode = encoding[0];
        match opcode {
            0x00 => implied(f, "brk", encoding),
            0x01 => indx(f, "ora", encoding),
            0x05 => zp(f, "ora", encoding),
            0x06 => zp(f, "asl", encoding),
            0x08 => implied(f, "php", encoding),
            0x09 => imm(f, "ora", encoding),
            0x0a => acc(f, "asl", encoding),
            0x0d => abs(f, "ora", encoding),
            0x0e => abs(f, "asl", encoding),
            0x10 => relative(f, "bpl", encoding),
            0x11 => indy(f, "ora", encoding),
            0x15 => zpx(f, "ora", encoding),
            0x16 => zpx(f, "asl", encoding),
            0x18 => implied(f, "clc", encoding),
            0x19 => absy(f, "ora", encoding),
            0x1d => absx(f, "ora", encoding),
            0x1e => absx(f, "asl", encoding),
            0x20 => abs(f, "jsr", encoding),
            0x21 => indx(f, "and", encoding),
            0x24 => zp(f, "bit", encoding),
            0x25 => zp(f, "and", encoding),
            0x26 => zp(f, "rol", encoding),
            0x28 => implied(f, "plp", encoding),
            0x29 => imm(f, "and", encoding),
            0x2a => acc(f, "rol", encoding),
            0x2c => abs(f, "bit", encoding),
            0x2d => abs(f, "and", encoding),
            0x2e => abs(f, "rol", encoding),
            0x30 => relative(f, "bmi", encoding),
            0x31 => indy(f, "and", encoding),
            0x35 => zpx(f, "and", encoding),
            0x36 => zpx(f, "rol", encoding),
            0x38 => implied(f, "sec", encoding),
            0x39 => absy(f, "and", encoding),
            0x3d => absx(f, "and", encoding),
            0x3e => absx(f, "rol", encoding),
            0x40 => implied(f, "rti", encoding),
            0x41 => indx(f, "eor", encoding),
            0x45 => zp(f, "eor", encoding),
            0x46 => zp(f, "lsr", encoding),
            0x48 => implied(f, "pha", encoding),
            0x49 => imm(f, "eor", encoding),
            0x4a => acc(f, "lsr", encoding),
            0x4c => abs(f, "jmp", encoding),
            0x4d => abs(f, "eor", encoding),
            0x4e => abs(f, "lsr", encoding),
            0x50 => relative(f, "bvc", encoding),
            0x51 => indy(f, "eor", encoding),
            0x55 => zpx(f, "eor", encoding),
            0x56 => zpx(f, "lsr", encoding),
            0x58 => implied(f, "cli", encoding),
            0x59 => absy(f, "eor", encoding),
            0x5d => absx(f, "eor", encoding),
            0x5e => absx(f, "lsr", encoding),
            0x60 => implied(f, "rts", encoding),
            0x61 => indx(f, "adc", encoding),
            0x65 => zp(f, "adc", encoding),
            0x66 => zp(f, "ror", encoding),
            0x68 => implied(f, "pla", encoding),
            0x69 => imm(f, "adc", encoding),
            0x6a => acc(f, "ror", encoding),
            0x6c => ind(f, "jmp", encoding),
            0x6d => abs(f, "adc", encoding),
            0x6e => abs(f, "ror", encoding),
            0x70 => relative(f, "bvs", encoding),
            0x71 => indy(f, "adc", encoding),
            0x75 => zpx(f, "adc", encoding),
            0x76 => zpx(f, "ror", encoding),
            0x78 => implied(f, "sei", encoding),
            0x79 => absy(f, "adc", encoding),
            0x7d => absx(f, "adc", encoding),
            0x7e => absx(f, "ror", encoding),
            0x81 => indx(f, "sta", encoding),
            0x84 => zp(f, "sty", encoding),
            0x85 => zp(f, "sta", encoding),
            0x86 => zp(f, "stx", encoding),
            0x88 => implied(f, "dey", encoding),
            0x8a => implied(f, "txa", encoding),
            0x8c => abs(f, "sty", encoding),
            0x8d => abs(f, "sta", encoding),
            0x8e => abs(f, "stx", encoding),
            0x90 => relative(f, "bcc", encoding),
            0x91 => indy(f, "sta", encoding),
            0x94 => zpx(f, "sty", encoding),
            0x95 => zpx(f, "sta", encoding),
            0x96 => zpy(f, "stx", encoding),
            0x98 => implied(f, "tya", encoding),
            0x99 => absy(f, "sta", encoding),
            0x9a => implied(f, "txs", encoding),
            0x9d => absx(f, "sta", encoding),
            0xa0 => imm(f, "ldy", encoding),
            0xa1 => indx(f, "lda", encoding),
            0xa2 => imm(f, "ldx", encoding),
            0xa4 => zp(f, "ldy", encoding),
            0xa5 => zp(f, "lda", encoding),
            0xa6 => zp(f, "ldx", encoding),
            0xa8 => implied(f, "tay", encoding),
            0xa9 => imm(f, "lda", encoding),
            0xaa => implied(f, "tax", encoding),
            0xac => abs(f, "ldy", encoding),
            0xad => abs(f, "lda", encoding),
            0xae => abs(f, "ldx", encoding),
            0xb0 => relative(f, "bcs", encoding),
            0xb1 => indy(f, "lda", encoding),
            0xb4 => zpx(f, "ldy", encoding),
            0xb5 => zpx(f, "lda", encoding),
            0xb6 => zpy(f, "ldx", encoding),
            0xb8 => implied(f, "clv", encoding),
            0xb9 => absy(f, "lda", encoding),
            0xba => implied(f, "tsx", encoding),
            0xbc => absx(f, "ldy", encoding),
            0xbd => absx(f, "lda", encoding),
            0xbe => absy(f, "ldx", encoding),
            0xc0 => imm(f, "cpy", encoding),
            0xc1 => indx(f, "cmp", encoding),
            0xc4 => zp(f, "cpy", encoding),
            0xc5 => zp(f, "cmp", encoding),
            0xc6 => zp(f, "dec", encoding),
            0xc8 => implied(f, "iny", encoding),
            0xc9 => imm(f, "cmp", encoding),
            0xca => implied(f, "dex", encoding),
            0xcc => abs(f, "cpy", encoding),
            0xcd => abs(f, "cmp", encoding),
            0xce => abs(f, "dec", encoding),
            0xd0 => relative(f, "bne", encoding),
            0xd1 => indy(f, "cmp", encoding),
            0xd5 => zpx(f, "cmp", encoding),
            0xd6 => zpx(f, "dec", encoding),
            0xd8 => implied(f, "cld", encoding),
            0xd9 => absy(f, "cmp", encoding),
            0xdd => absx(f, "cmp", encoding),
            0xde => absx(f, "dec", encoding),
            0xe0 => imm(f, "cpx", encoding),
            0xe1 => indx(f, "sbc", encoding),
            0xe4 => zp(f, "cpx", encoding),
            0xe5 => zp(f, "sbc", encoding),
            0xe6 => zp(f, "inc", encoding),
            0xe8 => implied(f, "inx", encoding),
            0xe9 => imm(f, "sbc", encoding),
            0xea => implied(f, "nop", encoding),
            0xec => abs(f, "cpx", encoding),
            0xed => abs(f, "sbc", encoding),
            0xee => abs(f, "inc", encoding),
            0xf0 => relative(f, "beq", encoding),
            0xf1 => indy(f, "sbc", encoding),
            0xf5 => zpx(f, "sbc", encoding),
            0xf6 => zpx(f, "inc", encoding),
            0xf8 => implied(f, "sed", encoding),
            0xf9 => absy(f, "sbc", encoding),
            0xfd => absx(f, "sbc", encoding),
            0xfe => absx(f, "inc", encoding),
            _ => fallback(f, encoding),
        }
    }

    impl std::fmt::Display for Nmos6502Instruction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            match self.encoding[0] {
                0x9f => absy(f, "ahx", self.encoding),
                0x93 => indy(f, "ahx", self.encoding),

                0x4b => imm(f, "alr", self.encoding),

                0x0b => imm(f, "anc", self.encoding),
                0x2b => imm(f, "anc", self.encoding),

                0xcb => imm(f, "asx", self.encoding),

                0x6f => abs(f, "arr", self.encoding),
                0x6b => imm(f, "arr", self.encoding),

                0xdf => absx(f, "dcp", self.encoding),
                0xdb => absy(f, "dcp", self.encoding),
                0xcf => abs(f, "dcp", self.encoding),
                0xc3 => indx(f, "dcp", self.encoding),
                0xd3 => indy(f, "dcp", self.encoding),
                0xc7 => zp(f, "dcp", self.encoding),
                0xd7 => zpx(f, "dcp", self.encoding),

                0xef => abs(f, "isc", self.encoding),
                0xfb => absy(f, "isc", self.encoding),
                0xff => absx(f, "isc", self.encoding),
                0xe3 => indx(f, "isc", self.encoding),
                0xf3 => indy(f, "isc", self.encoding),
                0xe7 => zp(f, "isc", self.encoding),
                0xf7 => zpx(f, "isc", self.encoding),

                0xbb => absy(f, "las", self.encoding),

                0xaf => abs(f, "lax", self.encoding),
                0xbf => absy(f, "lax", self.encoding),
                0xab => imm(f, "lax", self.encoding),
                0xa3 => indx(f, "lax", self.encoding),
                0xb3 => indy(f, "lax", self.encoding),
                0xa7 => zp(f, "lax", self.encoding),
                0xb7 => zpy(f, "lax", self.encoding),

                0x2f => abs(f, "rla", self.encoding),
                0x3f => absx(f, "rla", self.encoding),
                0x3b => absy(f, "rla", self.encoding),
                0x23 => indx(f, "rla", self.encoding),
                0x33 => indy(f, "rla", self.encoding),
                0x27 => zp(f, "rla", self.encoding),
                0x37 => zpx(f, "rla", self.encoding),

                0x7b => absx(f, "rra", self.encoding),
                0x7f => absx(f, "rra", self.encoding),
                0x63 => indx(f, "rra", self.encoding),
                0x73 => indy(f, "rra", self.encoding),
                0x67 => zp(f, "rra", self.encoding),
                0x77 => zpx(f, "rra", self.encoding),

                0x8f => abs(f, "sax", self.encoding),
                0x83 => indx(f, "sax", self.encoding),
                0x87 => zp(f, "sax", self.encoding),
                0x97 => zpy(f, "sax", self.encoding),

                0x9e => absy(f, "shx", self.encoding),

                0x9c => absx(f, "shy", self.encoding),

                0x4f => abs(f, "sre", self.encoding),
                0x5f => absx(f, "sre", self.encoding),
                0x5b => absy(f, "sre", self.encoding),
                0x43 => indx(f, "sre", self.encoding),
                0x53 => indy(f, "sre", self.encoding),
                0x47 => zp(f, "sre", self.encoding),
                0x57 => zpx(f, "sre", self.encoding),

                0x0f => abs(f, "slo", self.encoding),
                0x1f => absx(f, "slo", self.encoding),
                0x1b => absy(f, "slo", self.encoding),
                0x03 => indx(f, "slo", self.encoding),
                0x13 => indy(f, "slo", self.encoding),
                0x07 => zp(f, "slo", self.encoding),
                0x17 => zpx(f, "slo", self.encoding),

                0x9b => absy(f, "tas", self.encoding),

                0x8b => imm(f, "xaa", self.encoding),
                _ => common_disassembly(f, self.encoding),
            }
        }
    }

    impl std::fmt::Display for Cmos6502Instruction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            match self.encoding[0] {
                0x72 => zpi(f, "adc", self.encoding),
                0x32 => zpi(f, "and", self.encoding),
                0xd2 => zpi(f, "cmp", self.encoding),
                0x52 => zpi(f, "eor", self.encoding),
                0xb2 => zpi(f, "lda", self.encoding),
                0x12 => zpi(f, "ora", self.encoding),
                0xf2 => zpi(f, "sbc", self.encoding),
                0x92 => zpi(f, "sta", self.encoding),

                0x04 => zp(f, "tsb", self.encoding),
                0x14 => zp(f, "trb", self.encoding),
                0x0c => abs(f, "tsb", self.encoding),
                0x1c => abs(f, "trb", self.encoding),

                0x3a => acc(f, "dec", self.encoding),
                0x1a => acc(f, "inc", self.encoding),

                0x34 => zpx(f, "bit", self.encoding),
                0x3c => absx(f, "bit", self.encoding),

                0xda => implied(f, "phx", self.encoding),
                0x5a => implied(f, "phy", self.encoding),
                0xfa => implied(f, "plx", self.encoding),
                0x7a => implied(f, "ply", self.encoding),

                0x64 => zp(f, "stz", self.encoding),
                0x74 => zpx(f, "stz", self.encoding),
                0x9c => abs(f, "stz", self.encoding),
                0x9e => absx(f, "stz", self.encoding),
                0x7c => absix(f, "jmp", self.encoding),
                0x80 => relative(f, "bra", self.encoding),
                0x89 => imm(f, "bit", self.encoding),

                _ => common_disassembly(f, self.encoding),
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn all_nmos_opcodes_have_disassembly() {
        use super::Nmos6502Instruction;
        use super::NMOS_OPCODES;

        for opcode in NMOS_OPCODES {
            let instruction = Nmos6502Instruction::new([opcode, 0, 0]);
            let dasm = format!("{}", instruction);
            if dasm.contains(".db") {
                panic!("No disassembly for {}", dasm);
            }
        }
    }

    #[test]
    fn ror_is_not_reva_compatible() {
        // Very early chips do not have the ROR instruction, so test the reva_compatible method
        // filters away any instruction where the disassembly contains the substring, "ror".
        use super::Nmos6502Instruction;
        use super::NMOS_OPCODES;
        use crate::Instruction;

        for opcode in NMOS_OPCODES {
            let instruction = Nmos6502Instruction::new([opcode, 0, 0]);
            let dasm = format!("{}", instruction);
            if dasm.contains("ror") {
                if instruction.reva_compatible() {
                    panic!("the reva_compatible method is returning true for {}, having opcode ${:02x}", instruction, instruction.encode()[0])
                }
            } else if !instruction.reva_compatible() {
                panic!(
                    "the reva_compatible method is returning false for {}, having opcode ${:02x}",
                    instruction,
                    instruction.encode()[0]
                )
            }
        }
    }

    #[test]
    fn nmos_instructions_present_on_cmos() {
        use super::Cmos6502Instruction;
        use super::Nmos6502Instruction;
        use super::NMOS_OPCODES;

        for opcode in NMOS_OPCODES {
            let nmos_instruction = Nmos6502Instruction::new([opcode, 0, 0]);
            let cmos_instruction = Cmos6502Instruction::new([opcode, 0, 0]);

            let nmos_dasm = format!("{}", nmos_instruction);
            let cmos_dasm = format!("{}", cmos_instruction);

            if nmos_instruction.cmos_compatible() {
                if nmos_dasm != cmos_dasm {
                    panic!(
                        "${:02x} encodes {} on NMOS but {} on CMOS",
                        opcode, nmos_dasm, cmos_dasm
                    );
                }
            } else if nmos_dasm == cmos_dasm {
                panic!(
                    "${:02x} encodes {} on both NMOS and CMOS, but cmos_compatible returns false",
                    opcode, nmos_dasm
                );
            }
        }
    }

    #[test]
    fn all_cmos_opcodes_have_disassembly() {
        use super::Cmos6502Instruction;
        use super::CMOS_OPCODES;

        for opcode in CMOS_OPCODES {
            let instruction = Cmos6502Instruction::new([opcode, 0, 0]);
            let dasm = format!("{}", instruction);
            if dasm.contains(".db") {
                panic!("No disassembly for {}", dasm);
            }
            if let Some((offset, _)) = dasm.match_indices(';').next() {
                if offset != 20 {
                    panic!(
                        "Disassembly for {} has semicolon at wrong index {}.",
                        dasm, offset
                    );
                }
            } else {
                panic!("Disassembly for {} has no semicolon", dasm);
            }
        }
    }

    #[test]
    fn variables_in_memory() {
        use super::Nmos6502Instruction;
        use super::VariablesInMemory;
        use crate::Candidate;
        let insn = Nmos6502Instruction {
            encoding: [0xa5, 0x45, 0x00],
        };
        let cand = Candidate::new(vec![insn]);
        let vars = VariablesInMemory::from(&cand);
        assert_eq!(vars.reads[0], 0x45);
    }
}
