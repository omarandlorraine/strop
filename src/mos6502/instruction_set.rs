//! A module representing the MOS 6502's instruction set in a way that facilitates its use by
//! strop.

use crate::Fixup;
use crate::Instruction;
use crate::Range;
use rand::random;

fn take_apart(
    cmos_insn: Cmos6502Instruction,
) -> Option<(
    mos6502::instruction::Instruction,
    mos6502::instruction::AddressingMode,
)> {
    use mos6502::Variant;
    mos6502::instruction::Nmos6502::decode(cmos_insn.encode()[0])
}

fn instruction_writes(
    i: Cmos6502Instruction,
    insn: (
        mos6502::instruction::Instruction,
        mos6502::instruction::AddressingMode,
    ),
) -> Option<u16> {
    use mos6502::instruction::AddressingMode;
    use mos6502::instruction::Instruction;

    let (instruction, operand) = insn;
    match (instruction, operand) {
        (Instruction::ADC, _) => None,
        (Instruction::ADCnd, _) => None,
        (Instruction::AND, _) => None,
        (Instruction::BIT, _) => None,
        (Instruction::BCC, _) => None,
        (Instruction::BCS, _) => None,
        (Instruction::BEQ, _) => None,
        (Instruction::BMI, _) => None,
        (Instruction::BPL, _) => None,
        (Instruction::BNE, _) => None,
        (Instruction::BRK, _) => None,
        (Instruction::BVC, _) => None,
        (Instruction::BVS, _) => None,
        (Instruction::CMP, _) => None,
        (Instruction::CPX, _) => None,
        (Instruction::CPY, _) => None,
        (Instruction::CLC, _) => None,
        (Instruction::CLD, _) => None,
        (Instruction::CLI, _) => None,
        (Instruction::SEC, _) => None,
        (Instruction::SED, _) => None,
        (Instruction::SEI, _) => None,
        (Instruction::CLV, _) => None,
        (Instruction::DEX, _) => None,
        (Instruction::DEY, _) => None,
        (Instruction::EOR, _) => None,
        (Instruction::INX, _) => None,
        (Instruction::INY, _) => None,
        (Instruction::JMP, _) => None,
        (Instruction::JSR, _) => None,
        (Instruction::LDA, _) => None,
        (Instruction::LDX, _) => None,
        (Instruction::LDY, _) => None,
        (Instruction::NOP, _) => None,
        (Instruction::PHA, _) => None,
        (Instruction::PHP, _) => None,
        (Instruction::PLA, _) => None,
        (Instruction::PLP, _) => None,
        (Instruction::RTI, _) => None,
        (Instruction::RTS, _) => None,
        (Instruction::SBC, _) => None,
        (Instruction::SBCnd, _) => None,
        (Instruction::TAY, _) => None,
        (Instruction::TAX, _) => None,
        (Instruction::TSX, _) => None,
        (Instruction::TXA, _) => None,
        (Instruction::TYA, _) => None,
        (Instruction::TXS, _) => None,
        (Instruction::ORA, _) => None,
        (
            Instruction::STA
            | Instruction::STX
            | Instruction::STY
            | Instruction::ASL
            | Instruction::DEC
            | Instruction::INC
            | Instruction::LSR
            | Instruction::ROL
            | Instruction::ROR,
            operand,
        ) => {
            let encoding = i.encode();
            let one = [encoding[1], 0];
            let two = [encoding[1], encoding[2]];
            match operand {
                AddressingMode::Absolute => Some(u16::from_le_bytes(two)),
                AddressingMode::AbsoluteX => Some(u16::from_le_bytes(two)),
                AddressingMode::AbsoluteY => Some(u16::from_le_bytes(two)),
                AddressingMode::Accumulator => None,
                AddressingMode::Immediate => None,
                AddressingMode::Indirect => None,
                AddressingMode::IndexedIndirectX => Some(u16::from_le_bytes(one)),
                AddressingMode::IndirectIndexedY => Some(u16::from_le_bytes(one)),
                AddressingMode::Implied => None,
                AddressingMode::Relative => None,
                AddressingMode::ZeroPage => Some(u16::from_le_bytes(one)),
                AddressingMode::ZeroPageX => Some(u16::from_le_bytes(one)),
                AddressingMode::ZeroPageY => Some(u16::from_le_bytes(one)),
            }
        }
    }
}

fn instruction_reads(
    i: Cmos6502Instruction,
    insn: (
        mos6502::instruction::Instruction,
        mos6502::instruction::AddressingMode,
    ),
) -> Option<u16> {
    use mos6502::instruction::AddressingMode;
    use mos6502::instruction::Instruction;

    let (instruction, operand) = insn;
    match (instruction, operand) {
        (Instruction::STA, _) => None,
        (Instruction::STX, _) => None,
        (Instruction::STY, _) => None,
        (Instruction::BCC, _) => None,
        (Instruction::BCS, _) => None,
        (Instruction::BEQ, _) => None,
        (Instruction::BMI, _) => None,
        (Instruction::BPL, _) => None,
        (Instruction::BNE, _) => None,
        (Instruction::BRK, _) => None,
        (Instruction::BVC, _) => None,
        (Instruction::BVS, _) => None,
        (Instruction::CLC, _) => None,
        (Instruction::CLD, _) => None,
        (Instruction::CLI, _) => None,
        (Instruction::SEC, _) => None,
        (Instruction::SED, _) => None,
        (Instruction::SEI, _) => None,
        (Instruction::CLV, _) => None,
        (Instruction::DEX, _) => None,
        (Instruction::DEY, _) => None,
        (Instruction::INX, _) => None,
        (Instruction::INY, _) => None,
        (Instruction::JMP, _) => None,
        (Instruction::JSR, _) => None,
        (Instruction::NOP, _) => None,
        (Instruction::PHA, _) => None,
        (Instruction::PHP, _) => None,
        (Instruction::PLA, _) => None,
        (Instruction::PLP, _) => None,
        (Instruction::RTI, _) => None,
        (Instruction::RTS, _) => None,
        (Instruction::TAY, _) => None,
        (Instruction::TAX, _) => None,
        (Instruction::TSX, _) => None,
        (Instruction::TXA, _) => None,
        (Instruction::TYA, _) => None,
        (Instruction::TXS, _) => None,
        (
            Instruction::ADC
            | Instruction::ADCnd
            | Instruction::AND
            | Instruction::ASL
            | Instruction::BIT
            | Instruction::CMP
            | Instruction::CPX
            | Instruction::CPY
            | Instruction::DEC
            | Instruction::EOR
            | Instruction::INC
            | Instruction::LDA
            | Instruction::LDX
            | Instruction::LDY
            | Instruction::LSR
            | Instruction::ROL
            | Instruction::ROR
            | Instruction::SBC
            | Instruction::SBCnd
            | Instruction::ORA,
            operand,
        ) => {
            let one = [i.0[1], 0];
            let two = [i.0[1], i.0[2]];
            match operand {
                AddressingMode::Absolute => Some(u16::from_le_bytes(two)),
                AddressingMode::AbsoluteX => Some(u16::from_le_bytes(two)),
                AddressingMode::AbsoluteY => Some(u16::from_le_bytes(two)),
                AddressingMode::Accumulator => None,
                AddressingMode::Immediate => None,
                AddressingMode::Indirect => None,
                AddressingMode::IndexedIndirectX => Some(u16::from_le_bytes(one)),
                AddressingMode::IndirectIndexedY => Some(u16::from_le_bytes(one)),
                AddressingMode::Implied => None,
                AddressingMode::Relative => None,
                AddressingMode::ZeroPage => Some(u16::from_le_bytes(one)),
                AddressingMode::ZeroPageX => Some(u16::from_le_bytes(one)),
                AddressingMode::ZeroPageY => Some(u16::from_le_bytes(one)),
            }
        }
    }
}

/// Takes an instruction, and returns another instruction with the same opcode, but a random
/// operand.
pub fn randomize_operand(instruction: Cmos6502Instruction) -> Cmos6502Instruction {
    let opcode = instruction.encode()[0];
    Cmos6502Instruction([opcode, random(), random()])
}

/// Takes an instruction, and increments the operand.
pub fn increment_operand(instruction: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
    let mut insn = instruction;
    insn.increment()
}

fn increment_opcode(instruction: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
    let opcode = instruction.encode()[0];

    let index = CMOS_OPCODES.iter().position(|&r| r == opcode)? + 1;
    if index >= CMOS_OPCODES.len() {
        return None;
    }

    Some(Cmos6502Instruction([CMOS_OPCODES[index], 0, 0]))
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
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Cmos6502Instruction([u8; 3]);

impl std::convert::TryFrom<&[u8]> for Cmos6502Instruction {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let temp = Self([value[0], 0, 0]);
        match temp.length() {
            1 => Ok(Self([value[0], 0, 0])),
            2 => Ok(Self([value[0], value[1], 0])),
            3 => Ok(Self([value[0], value[1], value[2]])),
            _ => Err(()),
        }
    }
}

impl Cmos6502Instruction {
    /// Returns the length of the instruction in bytes. And a 6502 instruction is always either 1,
    /// 2 or 3 bytes.
    pub fn length(&self) -> usize {
        match self.0[0] {
            0x04 | 0x64 | 0x74 | 0x72 | 0x32 | 0xd2 | 0x52 | 0xb2 | 0x12 | 0xf2 | 0x92 | 0x14 => 2,
            0x01 | 0x03 | 0x34 | 0x89 | 0x80 | 0x05..=0x07 | 0x09 | 0x0b | 0x10 | 0x11 | 0x13 => 2,
            0x0c | 0x1c | 0x3c | 0x3a | 0x7c | 0x0d | 0x0e | 0x0f | 0x19 | 0x1b | 0x1d..=0x20 => 3,
            0x15..=0x17 | 0x21 | 0x23..=0x27 | 0x29 | 0x2b | 0x30 | 0x31 | 0x33 | 0x35..=0x37 => 2,
            0xda | 0x5a | 0xfa | 0x7a | 0x1a | 0x00 | 0x08 | 0x0a | 0x18 | 0x28 | 0x2a | 0x38 => 1,
            0x2c..=0x2f | 0x39 | 0x3b | 0x3d..=0x3f | 0x4c | 0x4d..=0x4f | 0x59 | 0x5b | 0x79 => 3,
            0x41 | 0x43 | 0x45..=0x47 | 0x49 | 0x4b | 0x50 | 0x51 | 0x53 | 0x55..=0x57 | 0x61 => 2,
            0x63 | 0x65..=0x67 | 0x69 | 0x6b | 0x70 | 0x71 | 0x73 | 0x75..=0x77 | 0x81 | 0x83 => 2,
            0x40 | 0x48 | 0x4a | 0x58 | 0x60 | 0x68 | 0x6a | 0x78 | 0x88 | 0x8a | 0x98 | 0x9a => 1,
            0x6c | 0x6d..=0x6f | 0x5d..=0x5f | 0x7b | 0x7d..=0x7f | 0x8c..=0x8f | 0x9b..=0x9f => 3,
            0x84..=0x87 | 0x8b | 0x90 | 0x91 | 0x93..=0x97 | 0xa0..=0xa7 | 0xa9 | 0xab | 0xb0 => 2,
            0xb1 | 0xb3..=0xb7 | 0xc0 | 0xc1 | 0xc3..=0xc7 | 0xc9 | 0xcb | 0xd0 | 0xd1 | 0xd3 => 2,
            0x99 | 0xac | 0xad..=0xaf | 0xb9 | 0xbb..=0xbf | 0xcc..=0xcf | 0xd9 | 0xdb | 0xec => 3,
            0xd5..=0xd7 | 0xe0 | 0xe1 | 0xe3..=0xe7 | 0xe9 | 0xf0 | 0xf1 | 0xf3 | 0xf5..=0xf7 => 2,
            0xa8 | 0xaa | 0xb8 | 0xba | 0xc8 | 0xca | 0xd8 | 0xe8 | 0xea | 0xf8 => 1,
            0xdd..=0xdf | 0xed | 0xee | 0xef | 0xf9 | 0xfb | 0xfd..=0xff => 3,
            _ => 0,
        }
    }

    /// Returns a new Cmos6502Instruction, from the encoding
    pub fn new(encoding: [u8; 3]) -> Self {
        Self(encoding)
    }

    fn reads_from(self) -> Option<u16> {
        instruction_reads(self, take_apart(self)?)
    }

    fn writes_to(self) -> Option<u16> {
        instruction_writes(self, take_apart(self)?)
    }
}

#[derive(Clone, Debug)]
pub struct Writes<T: Range<u16> + std::fmt::Debug>(pub T);

impl<T: Range<u16> + std::fmt::Debug> Fixup<Cmos6502Instruction> for Writes<T> {
    fn check(&self, insn: Cmos6502Instruction) -> bool {
        if let Some(addr) = insn.writes_to() {
            self.0.check(addr)
        } else {
            // this instruction doesn't read from anywhere, so there's nothing to fix up
            false
        }
    }

    fn random(&self, insn: Cmos6502Instruction) -> Cmos6502Instruction {
        let [lo, hi] = self.0.random().to_le_bytes();
        Cmos6502Instruction([insn.0[0], lo, hi])
    }

    fn next(&self, insn: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
        if let Some(addr) = insn.reads_from() {
            if let Some(addr) = self.0.next(addr) {
                let [lo, hi] = addr.to_le_bytes();
                Some(Cmos6502Instruction([insn.0[0], lo, hi]))
            } else {
                self.next(increment_opcode(insn)?)
            }
        } else {
            self.next(increment_opcode(insn)?)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reads<T: Range<u16> + std::fmt::Debug>(pub T);

impl<T: Range<u16> + std::fmt::Debug> Fixup<Cmos6502Instruction> for Reads<T> {
    fn check(&self, insn: Cmos6502Instruction) -> bool {
        if let Some(addr) = insn.reads_from() {
            self.0.check(addr)
        } else {
            // this instruction doesn't read from anywhere, so there's nothing to fix up
            false
        }
    }

    fn random(&self, insn: Cmos6502Instruction) -> Cmos6502Instruction {
        let [lo, hi] = self.0.random().to_le_bytes();
        Cmos6502Instruction([insn.0[0], lo, hi])
    }

    fn next(&self, insn: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
        if let Some(addr) = insn.reads_from() {
            if let Some(addr) = self.0.next(addr) {
                let [lo, hi] = addr.to_le_bytes();
                Some(Cmos6502Instruction([insn.0[0], lo, hi]))
            } else {
                self.next(increment_opcode(insn)?)
            }
        } else {
            self.next(increment_opcode(insn)?)
        }
    }
}

impl Instruction for Cmos6502Instruction {
    fn random() -> Self {
        use rand::prelude::SliceRandom;
        Self([
            *CMOS_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            random(),
            random(),
        ])
    }

    fn encode(self) -> Vec<u8> {
        match self.length() {
            1 => vec![self.0[0]],
            2 => vec![self.0[0], self.0[1]],
            3 => vec![self.0[0], self.0[1], self.0[2]],
            _ => panic!(),
        }
    }

    fn mutate(&mut self) {
        if random() {
            use rand::prelude::SliceRandom;
            self.0[0] = *CMOS_OPCODES.choose(&mut rand::thread_rng()).unwrap();
        } else {
            self.0[1] = random();
            self.0[2] = random();
        }
    }

    fn first() -> Self {
        Self([0, 0, 0])
    }

    fn increment(&mut self) -> Option<Self> {
        let length = self.length();

        fn next_opcode(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            increment_opcode(*insn)
        }

        fn next2(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            insn.0[1] = insn.0[1].wrapping_add(1); // ready for next call
            if insn.0[1] == 0 {
                next_opcode(insn)
            } else {
                Some(Cmos6502Instruction::new(insn.0))
            }
        }

        fn next3(insn: &mut Cmos6502Instruction) -> Option<Cmos6502Instruction> {
            let operand = u16::from_le_bytes([insn.0[1], insn.0[2]]);
            if let Some(new_operand) = operand.checked_add(1) {
                let le_bytes = new_operand.to_le_bytes();
                insn.0[1] = le_bytes[0];
                insn.0[2] = le_bytes[1];
                Some(insn.clone())
            } else {
                next_opcode(insn)
            }
        }

        match length {
            1 => next_opcode(self),
            2 => next2(self),
            3 => next3(self),
            _ => unreachable!(
                "Opcode {}, whose opcode is ${:02x}, has length {}",
                self, self.0[0], length
            ),
        }
    }
}

mod disassembly {
    use super::Cmos6502Instruction;

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

    impl std::fmt::Display for Cmos6502Instruction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            match self.0[0] {
                0x72 => zpi(f, "adc", self.0),
                0x32 => zpi(f, "and", self.0),
                0xd2 => zpi(f, "cmp", self.0),
                0x52 => zpi(f, "eor", self.0),
                0xb2 => zpi(f, "lda", self.0),
                0x12 => zpi(f, "ora", self.0),
                0xf2 => zpi(f, "sbc", self.0),
                0x92 => zpi(f, "sta", self.0),

                0x04 => zp(f, "tsb", self.0),
                0x14 => zp(f, "trb", self.0),
                0x0c => abs(f, "tsb", self.0),
                0x1c => abs(f, "trb", self.0),

                0x3a => acc(f, "dec", self.0),
                0x1a => acc(f, "inc", self.0),

                0x34 => zpx(f, "bit", self.0),
                0x3c => absx(f, "bit", self.0),

                0xda => implied(f, "phx", self.0),
                0x5a => implied(f, "phy", self.0),
                0xfa => implied(f, "plx", self.0),
                0x7a => implied(f, "ply", self.0),

                0x64 => zp(f, "stz", self.0),
                0x74 => zpx(f, "stz", self.0),
                0x9c => abs(f, "stz", self.0),
                0x9e => absx(f, "stz", self.0),
                0x7c => absix(f, "jmp", self.0),
                0x80 => relative(f, "bra", self.0),
                0x89 => imm(f, "bit", self.0),

                _ => common_disassembly(f, self.0),
            }
        }
    }
}

#[cfg(test)]
mod test {
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
    fn indx_reads_from() {
        use crate::mos6502::instruction_set::Cmos6502Instruction;
        assert_eq!(Cmos6502Instruction([0xa5, 0, 0]).reads_from(), Some(0));
        assert_eq!(Cmos6502Instruction([0x1, 1, 0]).reads_from(), Some(1));
        assert_eq!(Cmos6502Instruction([0x1, 5, 0]).reads_from(), Some(5));
    }

    #[test]
    fn reads_from() {
        use crate::mos6502::instruction_set::Cmos6502Instruction;
        use crate::mos6502::instruction_set::Reads;
        use crate::Fixup;
        assert!(!Reads(vec![0]).check(Cmos6502Instruction([0xa5, 0, 0])));
        assert!(!Reads(vec![1]).check(Cmos6502Instruction([0x1, 1, 0])));
        assert!(Reads(vec![5]).check(Cmos6502Instruction([0x1, 1, 0])));
    }
}
