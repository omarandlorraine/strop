//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![allow(dead_code)]

use crate::instruction::Instruction;
use crate::randomly;
use rand::prelude::SliceRandom;
use rand::random;
use yaxpeax_6502::Instruction as YaxpeaxInstruction;
use yaxpeax_6502::{Opcode, Operand};
use yaxpeax_arch::Decoder;

use yaxpeax_arch::U8Reader;

fn instruction_length(op: Operand) -> usize {
    match op {
        Operand::Accumulator | Operand::Implied => 1,
        Operand::Relative(_) => 2,
        Operand::Immediate(_) => 2,
        Operand::IndirectYIndexed(_) | Operand::XIndexedIndirect(_) => 2,
        Operand::ZeroPage(_) | Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => 2,
        Operand::Absolute(_) | Operand::AbsoluteX(_) | Operand::AbsoluteY(_) => 3,
        Operand::Indirect(_) => 3,
    }
}

pub fn decode(machine_code: &[u8]) -> Option<(Opcode, Operand)> {
    let mut inst = YaxpeaxInstruction::default();
    let decoder = yaxpeax_6502::InstDecoder;
    let mut reader = U8Reader::new(machine_code);
    if decoder.decode_into(&mut inst, &mut reader).is_ok() {
        Some((inst.opcode, inst.operand))
    } else {
        None
    }
}

/// Represents a 6502 Instruction
#[derive(Clone, Copy, Debug)]
pub struct Instruction6502 {
    pub opcode: Opcode,
    pub operand: Operand,
}

impl Instruction6502 {
    /// returns true iff the instruction reads the X register
    pub fn reads_x(&self) -> bool {
        match (self.opcode, self.operand) {
            (_, Operand::XIndexedIndirect(_)) => true,
            (_, Operand::ZeroPageX(_)) => true,
            (_, Operand::AbsoluteX(_)) => true,
            (Opcode::TXA, _) => true,
            (Opcode::STX, _) => true,
            (Opcode::CPX, _) => true,
            (Opcode::DEX, _) => true,
            (Opcode::INX, _) => true,
            (Opcode::TXS, _) => true,
            (_, _) => false,
        }
    }

    /// returns true if the instruction sets the X register
    pub fn sets_x(&self) -> bool {
        match self.opcode {
            Opcode::TAX => true,
            Opcode::LDX => true,
            _ => false,
        }
    }

    /// returns true iff the instruction is a conditional branch
    fn is_branch(&self) -> bool {
        match self.opcode {
            Opcode::BCC => true,
            Opcode::BCS => true,
            Opcode::BEQ => true,
            Opcode::BMI => true,
            Opcode::BNE => true,
            Opcode::BPL => true,
            Opcode::BVC => true,
            Opcode::BVS => true,
            _ => false,
        }
    }

    /// returns true iff the instruction is a forward branch
    fn is_forward_branch(&self) -> bool {
        use yaxpeax_6502::Operand::Relative;
        if !self.is_branch() {
            // Not a branch, therefore not a forward branch.
            return false;
        }

        match self.operand {
            Relative(offset) => offset & 0x80 != 0x00,
            _ => panic!(),
        }
    }

    /// returns true iff the instruction is a backward branch
    fn is_backward_branch(&self) -> bool {
        use yaxpeax_6502::Operand::Relative;
        if !self.is_branch() {
            // Not a branch, therefore not a backward branch.
            return false;
        }

        match self.operand {
            Relative(offset) => offset & 0x80 == 0x00,
            _ => panic!(),
        }
    }

    pub fn avoid_rorbug(&self) -> bool {
        //! Returns false if the instruction exercises the ROR bug.
        //! Early revisions of the MOS 6502 do not have this instruction
        !matches!(self.opcode, Opcode::ROR)
    }
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (opcode, operand) = (self.opcode, self.operand);
        let st = format!("{}", opcode).to_lowercase();

        match operand {
            Operand::Implied => write!(f, "{}", st),
            Operand::Accumulator => write!(f, "{} a", st),
            Operand::Immediate(val) => write!(f, "{} #${:02x}", st, val),
            Operand::ZeroPage(addr) => write!(f, "{} ${:02x}", st, addr),
            Operand::ZeroPageX(addr) => write!(f, "{} ${:02x},x", st, addr),
            Operand::ZeroPageY(addr) => write!(f, "{} ${:02x},y", st, addr),
            Operand::Absolute(addr) => write!(f, "{} ${:04x}", st, addr),
            Operand::AbsoluteX(addr) => write!(f, "{} ${:04x},x", st, addr),
            Operand::AbsoluteY(addr) => write!(f, "{} ${:04x},y", st, addr),
            Operand::Indirect(addr) => write!(f, "{} (${:04x})", st, addr),
            Operand::IndirectYIndexed(addr) => write!(f, "{} (${:02x}),y", st, addr),
            Operand::XIndexedIndirect(addr) => write!(f, "{} (${:02x},x)", st, addr),
            Operand::Relative(offs) => write!(f, "{} ${:02x}", st, offs), // todo
        }
    }
}

impl Instruction for Instruction6502 {
    fn length(&self) -> usize {
        match self.operand {
            Operand::Accumulator | Operand::Implied => 1,
            Operand::Relative(_) => 2,
            Operand::Immediate(_) => 2,
            Operand::IndirectYIndexed(_) | Operand::XIndexedIndirect(_) => 2,
            Operand::ZeroPage(_) | Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => 2,
            Operand::Absolute(_) | Operand::AbsoluteX(_) | Operand::AbsoluteY(_) => 3,
            Operand::Indirect(_) => 3,
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        loop {
            let rand: Vec<u8> = vec![random(), random(), random()];
            if let Some((opcode, operand)) = decode(&rand) {
                return Instruction6502 { opcode, operand };
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match (self.opcode, self.operand) {
            (Opcode::BRK, Operand::Implied) => vec![0x00],
            (Opcode::ORA, Operand::XIndexedIndirect(val)) => vec![0x01, val],
            (Opcode::ORA, Operand::ZeroPage(val)) => vec![0x05, val],
            (Opcode::ASL, Operand::ZeroPage(val)) => vec![0x06, val],
            (Opcode::PHP, Operand::Implied) => vec![0x08],
            (Opcode::ORA, Operand::Immediate(val)) => vec![0x09, val],
            (Opcode::ASL, Operand::Accumulator) => vec![0x0a],
            (Opcode::ORA, Operand::Absolute(addr)) => {
                vec![0x0d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ASL, Operand::Absolute(addr)) => {
                vec![0x0e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BPL, Operand::Relative(val)) => vec![0x10, val],
            (Opcode::ORA, Operand::IndirectYIndexed(val)) => vec![0x11, val],
            (Opcode::ORA, Operand::ZeroPageX(val)) => vec![0x15, val],
            (Opcode::ASL, Operand::ZeroPageX(val)) => vec![0x16, val],
            (Opcode::CLC, Operand::Implied) => vec![0x18],
            (Opcode::ORA, Operand::AbsoluteY(addr)) => {
                vec![0x19, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ORA, Operand::AbsoluteX(addr)) => {
                vec![0x1d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ASL, Operand::AbsoluteX(addr)) => {
                vec![0x1e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::JSR, Operand::Absolute(addr)) => {
                vec![0x20, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::AND, Operand::XIndexedIndirect(val)) => vec![0x21, val],
            (Opcode::BIT, Operand::ZeroPage(val)) => vec![0x24, val],
            (Opcode::AND, Operand::ZeroPage(val)) => vec![0x25, val],
            (Opcode::ROL, Operand::ZeroPage(val)) => vec![0x26, val],
            (Opcode::PLP, Operand::Implied) => vec![0x28],
            (Opcode::AND, Operand::Immediate(val)) => vec![0x29, val],
            (Opcode::ROL, Operand::Accumulator) => vec![0x2a],
            (Opcode::BIT, Operand::Absolute(addr)) => {
                vec![0x2c, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::AND, Operand::Absolute(addr)) => {
                vec![0x2d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ROL, Operand::Absolute(addr)) => {
                vec![0x2e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::BMI, Operand::Relative(val)) => vec![0x30, val],
            (Opcode::AND, Operand::IndirectYIndexed(val)) => vec![0x31, val],
            (Opcode::AND, Operand::ZeroPageX(val)) => vec![0x35, val],
            (Opcode::ROL, Operand::ZeroPageX(val)) => vec![0x36, val],
            (Opcode::SEC, Operand::Implied) => vec![0x38],
            (Opcode::AND, Operand::AbsoluteY(addr)) => {
                vec![0x39, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::AND, Operand::AbsoluteX(addr)) => {
                vec![0x3d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ROL, Operand::AbsoluteX(addr)) => {
                vec![0x3e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::RTI, Operand::Implied) => vec![0x40],
            (Opcode::EOR, Operand::XIndexedIndirect(val)) => vec![0x41, val],
            (Opcode::EOR, Operand::ZeroPage(val)) => vec![0x45, val],
            (Opcode::LSR, Operand::ZeroPage(val)) => vec![0x46, val],
            (Opcode::PHA, Operand::Implied) => vec![0x48],
            (Opcode::EOR, Operand::Immediate(val)) => vec![0x49, val],
            (Opcode::LSR, Operand::Accumulator) => vec![0x4a],
            (Opcode::JMP, Operand::Absolute(addr)) => {
                vec![0x4c, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::EOR, Operand::Absolute(addr)) => {
                vec![0x4d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LSR, Operand::Absolute(addr)) => {
                vec![0x4e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BVC, Operand::Relative(val)) => vec![0x50, val],
            (Opcode::EOR, Operand::IndirectYIndexed(val)) => vec![0x51, val],
            (Opcode::EOR, Operand::ZeroPageX(val)) => vec![0x55, val],
            (Opcode::LSR, Operand::ZeroPageX(val)) => vec![0x56, val],
            (Opcode::CLI, Operand::Implied) => vec![0x58],
            (Opcode::EOR, Operand::AbsoluteY(addr)) => {
                vec![0x59, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::EOR, Operand::AbsoluteX(addr)) => {
                vec![0x5d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LSR, Operand::AbsoluteX(addr)) => {
                vec![0x5e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::RTS, Operand::Implied) => vec![0x60],
            (Opcode::ADC, Operand::XIndexedIndirect(val)) => vec![0x61, val],
            (Opcode::ADC, Operand::ZeroPage(val)) => vec![0x65, val],
            (Opcode::ROR, Operand::ZeroPage(val)) => vec![0x66, val],
            (Opcode::PLA, Operand::Implied) => vec![0x68],
            (Opcode::ADC, Operand::Immediate(val)) => vec![0x69, val],
            (Opcode::ROR, Operand::Accumulator) => vec![0x6a],
            (Opcode::JMP, Operand::Indirect(addr)) => {
                vec![0x6c, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ADC, Operand::Absolute(addr)) => {
                vec![0x6d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ROR, Operand::Absolute(addr)) => {
                vec![0x6e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::BVS, Operand::Relative(val)) => vec![0x70, val],
            (Opcode::ADC, Operand::IndirectYIndexed(val)) => vec![0x71, val],
            (Opcode::ADC, Operand::ZeroPageX(val)) => vec![0x75, val],
            (Opcode::ROR, Operand::ZeroPageX(val)) => vec![0x76, val],
            (Opcode::SEI, Operand::Implied) => vec![0x78],
            (Opcode::ADC, Operand::AbsoluteY(addr)) => {
                vec![0x79, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ADC, Operand::AbsoluteX(addr)) => {
                vec![0x7d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::ROR, Operand::AbsoluteX(addr)) => {
                vec![0x7e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::STA, Operand::XIndexedIndirect(val)) => vec![0x81, val],
            (Opcode::STY, Operand::ZeroPage(val)) => vec![0x84, val],
            (Opcode::STA, Operand::ZeroPage(val)) => vec![0x85, val],
            (Opcode::STX, Operand::ZeroPage(val)) => vec![0x86, val],
            (Opcode::DEY, Operand::Implied) => vec![0x88],
            (Opcode::TXA, Operand::Implied) => vec![0x8a],
            (Opcode::STY, Operand::Absolute(addr)) => {
                vec![0x8c, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::STA, Operand::Absolute(addr)) => {
                vec![0x8d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::STX, Operand::Absolute(addr)) => {
                vec![0x8e, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BCC, Operand::Relative(val)) => vec![0x90, val],
            (Opcode::STA, Operand::IndirectYIndexed(val)) => vec![0x91, val],
            (Opcode::STY, Operand::ZeroPageX(val)) => vec![0x94, val],
            (Opcode::STA, Operand::ZeroPageX(val)) => vec![0x95, val],
            (Opcode::STX, Operand::ZeroPageY(val)) => vec![0x96, val],
            (Opcode::TYA, Operand::Implied) => vec![0x98],
            (Opcode::STA, Operand::AbsoluteY(addr)) => {
                vec![0x99, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::TXS, Operand::Implied) => vec![0x9a],
            (Opcode::STA, Operand::AbsoluteX(addr)) => {
                vec![0x9d, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::LDY, Operand::Immediate(val)) => vec![0xa0, val],
            (Opcode::LDA, Operand::XIndexedIndirect(val)) => vec![0xa1, val],
            (Opcode::LDX, Operand::Immediate(val)) => vec![0xa2, val],
            (Opcode::LDY, Operand::ZeroPage(val)) => vec![0xa4, val],
            (Opcode::LDA, Operand::ZeroPage(val)) => vec![0xa5, val],
            (Opcode::LDX, Operand::ZeroPage(val)) => vec![0xa6, val],
            (Opcode::TAY, Operand::Implied) => vec![0xa8],
            (Opcode::LDA, Operand::Immediate(val)) => vec![0xa9, val],
            (Opcode::TAX, Operand::Implied) => vec![0xaa],
            (Opcode::LDY, Operand::Absolute(addr)) => {
                vec![0xac, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LDA, Operand::Absolute(addr)) => {
                vec![0xad, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LDX, Operand::Absolute(addr)) => {
                vec![0xae, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BCS, Operand::Relative(val)) => vec![0xb0, val],
            (Opcode::LDA, Operand::IndirectYIndexed(val)) => vec![0xb1, val],
            (Opcode::LDY, Operand::ZeroPageX(val)) => vec![0xb4, val],
            (Opcode::LDA, Operand::ZeroPageX(val)) => vec![0xb5, val],
            (Opcode::LDX, Operand::ZeroPageY(val)) => vec![0xb6, val],
            (Opcode::CLV, Operand::Implied) => vec![0xb8],
            (Opcode::LDA, Operand::AbsoluteY(addr)) => {
                vec![0xb9, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::TSX, Operand::Implied) => vec![0xba],
            (Opcode::LDY, Operand::AbsoluteX(addr)) => {
                vec![0xbc, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LDA, Operand::AbsoluteX(addr)) => {
                vec![0xbd, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::LDX, Operand::AbsoluteY(addr)) => {
                vec![0xbe, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::CPY, Operand::Immediate(val)) => vec![0xc0, val],
            (Opcode::CMP, Operand::XIndexedIndirect(val)) => vec![0xc1, val],
            (Opcode::CPY, Operand::ZeroPage(val)) => vec![0xc4, val],
            (Opcode::CMP, Operand::ZeroPage(val)) => vec![0xc5, val],
            (Opcode::DEC, Operand::ZeroPage(val)) => vec![0xc6, val],
            (Opcode::INY, Operand::Implied) => vec![0xc8],
            (Opcode::CMP, Operand::Immediate(val)) => vec![0xc9, val],
            (Opcode::DEX, Operand::Implied) => vec![0xca],
            (Opcode::CPY, Operand::Absolute(addr)) => {
                vec![0xcc, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::CMP, Operand::Absolute(addr)) => {
                vec![0xcd, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::DEC, Operand::Absolute(addr)) => {
                vec![0xce, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BNE, Operand::Relative(val)) => vec![0xd0, val],
            (Opcode::CMP, Operand::IndirectYIndexed(val)) => vec![0xd1, val],
            (Opcode::CMP, Operand::ZeroPageX(val)) => vec![0xd5, val],
            (Opcode::DEC, Operand::ZeroPageX(val)) => vec![0xd6, val],
            (Opcode::CLD, Operand::Implied) => vec![0xd8],
            (Opcode::CMP, Operand::AbsoluteY(addr)) => {
                vec![0xd9, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::CMP, Operand::AbsoluteX(addr)) => {
                vec![0xdd, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::DEC, Operand::AbsoluteX(addr)) => {
                vec![0xde, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::CPX, Operand::Immediate(val)) => vec![0xe0, val],
            (Opcode::SBC, Operand::XIndexedIndirect(addr)) => vec![0xe1, addr],
            (Opcode::CPX, Operand::ZeroPage(val)) => vec![0xe4, val],
            (Opcode::SBC, Operand::ZeroPage(val)) => vec![0xe5, val],
            (Opcode::INC, Operand::ZeroPage(val)) => vec![0xe6, val],
            (Opcode::INX, Operand::Implied) => vec![0xe8],
            (Opcode::SBC, Operand::Immediate(val)) => vec![0xe9, val],
            (Opcode::NOP, Operand::Implied) => vec![0xea],
            (Opcode::CPX, Operand::Absolute(addr)) => {
                vec![0xec, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::SBC, Operand::Absolute(addr)) => {
                vec![0xed, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::INC, Operand::Absolute(addr)) => {
                vec![0xee, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }

            (Opcode::BEQ, Operand::Relative(val)) => vec![0xf0, val],
            (Opcode::SBC, Operand::IndirectYIndexed(val)) => vec![0xf1, val],
            (Opcode::SBC, Operand::ZeroPageX(val)) => vec![0xf5, val],
            (Opcode::INC, Operand::ZeroPageX(val)) => vec![0xf6, val],
            (Opcode::SED, Operand::Implied) => vec![0xf8],
            (Opcode::SBC, Operand::AbsoluteY(addr)) => {
                vec![0xf9, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::SBC, Operand::AbsoluteX(addr)) => {
                vec![0xfd, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (Opcode::INC, Operand::AbsoluteX(addr)) => {
                vec![0xfe, addr.to_le_bytes()[0], addr.to_le_bytes()[1]]
            }
            (_, _) => panic!(),
        }
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        Box::new(self.to_bytes().into_iter())
    }

    fn perm_bb(&self) -> bool {
        !matches!(
            self.opcode,
            Opcode::BCC
                | Opcode::BCS
                | Opcode::BEQ
                | Opcode::BMI
                | Opcode::BNE
                | Opcode::BPL
                | Opcode::BVC
                | Opcode::BVS
                | Opcode::JMP
                | Opcode::JSR
                | Opcode::RTS
                | Opcode::RTI
        )
    }

    fn mutate_operand(&mut self) {
        // Pick another opcode having the same addressing mode
        self.operand = match self.operand {
            Operand::Implied => Operand::Implied,
            Operand::Accumulator => Operand::Accumulator,
            Operand::Immediate(v) => {
                // try incrementing the value, decrementing it, or flipping a random bit
                let random_bit = *vec![0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80]
                    .choose(&mut rand::thread_rng())
                    .unwrap();
                Operand::Immediate(randomly!(
                    { v.wrapping_add(1) }
                    { v.wrapping_sub(1) }
                    { v ^ random_bit }
                ))
            }
            Operand::ZeroPage(_) => Operand::ZeroPage(random()),
            Operand::ZeroPageX(_) => Operand::ZeroPageX(random()),
            Operand::ZeroPageY(_) => Operand::ZeroPageY(random()),
            Operand::Absolute(_) => Operand::Absolute(random()),
            Operand::AbsoluteX(_) => Operand::AbsoluteX(random()),
            Operand::AbsoluteY(_) => Operand::AbsoluteY(random()),
            Operand::Indirect(_) => Operand::Indirect(random()),
            Operand::IndirectYIndexed(_) => Operand::IndirectYIndexed(random()),
            Operand::XIndexedIndirect(_) => Operand::XIndexedIndirect(random()),
            Operand::Relative(_) => Operand::Relative(random()),
        }
    }

    fn mutate_opcode(&mut self) {
        // Pick another opcode having the same addressing mode
        use crate::mos6502::data::*;

        self.opcode = match self.operand {
            Operand::Implied => *IMP_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Accumulator => *ACC_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Immediate(_) => *IMM_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPage(_) => *ZP_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPageX(_) => *ZPX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPageY(_) => *ZPY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Absolute(_) => *ABS_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::AbsoluteX(_) => *ABSX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::AbsoluteY(_) => *ABSY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Indirect(_) => *IND_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::IndirectYIndexed(_) => *INDX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::XIndexedIndirect(_) => *INDY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Relative(_) => *REL_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
        };
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::mos6502::data::ABSX_OPCODES;
    use crate::mos6502::data::ABSY_OPCODES;
    use crate::mos6502::data::ABS_OPCODES;
    use crate::mos6502::data::ACC_OPCODES;
    use crate::mos6502::data::IMM_OPCODES;
    use crate::mos6502::data::IMP_OPCODES;
    use crate::mos6502::data::INDX_OPCODES;
    use crate::mos6502::data::INDY_OPCODES;
    use crate::mos6502::data::IND_OPCODES;
    use crate::mos6502::data::REL_OPCODES;
    use crate::mos6502::data::ZPX_OPCODES;
    use crate::mos6502::data::ZPY_OPCODES;
    use crate::mos6502::data::ZP_OPCODES;

    use crate::mos6502::Instruction6502;
    use yaxpeax_6502::Operand;

    #[test]
    fn new_instructions() {
        for _i in 0..50000 {
            let insn = Instruction6502::new();

            insn.length();
            insn.as_bytes();

            let _disasm = format!("{}", insn);

            let opcode = insn.opcode;

            match (insn.opcode, insn.operand) {
                (_, Operand::Implied) => assert!(
                    IMP_OPCODES.contains(&insn.opcode),
                    "{} should be in IMP_OPCODES",
                    opcode
                ),
                (_, Operand::Accumulator) => assert!(
                    ACC_OPCODES.contains(&insn.opcode),
                    "{} should be in ACC_OPCODES",
                    opcode
                ),
                (_, Operand::Immediate(_)) => assert!(
                    IMM_OPCODES.contains(&insn.opcode),
                    "{} should be in IMM_OPCODES",
                    opcode
                ),
                (_, Operand::Absolute(_)) => assert!(
                    ABS_OPCODES.contains(&insn.opcode),
                    "{} should be in ABS_OPCODES",
                    opcode
                ),
                (_, Operand::AbsoluteX(_)) => assert!(
                    ABSX_OPCODES.contains(&insn.opcode),
                    "{} should be in ABSX_OPCODES",
                    opcode
                ),
                (_, Operand::AbsoluteY(_)) => assert!(
                    ABSY_OPCODES.contains(&insn.opcode),
                    "{} should be in ABSY_OPCODES",
                    opcode
                ),
                (_, Operand::Indirect(_)) => assert!(
                    IND_OPCODES.contains(&insn.opcode),
                    "{} should be in IND_OPCODES",
                    opcode
                ),
                (_, Operand::XIndexedIndirect(_)) => assert!(
                    INDX_OPCODES.contains(&insn.opcode),
                    "{} should be in INDX_OPCODES",
                    opcode
                ),
                (_, Operand::IndirectYIndexed(_)) => assert!(
                    INDY_OPCODES.contains(&insn.opcode),
                    "{} should be in INDY_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPage(_)) => assert!(
                    ZP_OPCODES.contains(&insn.opcode),
                    "{} should be in ZP_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPageX(_)) => assert!(
                    ZPX_OPCODES.contains(&insn.opcode),
                    "{} should be in ZPX_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPageY(_)) => assert!(
                    ZPY_OPCODES.contains(&insn.opcode),
                    "{} should be in ZPY_OPCODES",
                    opcode
                ),
                (_, Operand::Relative(_)) => assert!(
                    REL_OPCODES.contains(&insn.opcode),
                    "{} should be in REL_OPCODES",
                    opcode
                ),
            }
        }
    }
}
