//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::instruction::Instruction;
use rand::prelude::SliceRandom;
use rand::random;
use yaxpeax_6502::Instruction as YaxpeaxInstruction;
use yaxpeax_6502::{Opcode, Operand};
use yaxpeax_arch::Decoder;
use yaxpeax_arch::U8Reader;

fn random_codepoint() -> u8 {
    // returns one random, valid opcode
    *vec![
        0x69, 0x65, 0x75, 0x6d, 0x7d, 0x79, 0x61, 0x71, 0x29, 0x25, 0x35, 0x2d, 0x3d, 0x39, 0x21,
        0x31, 0x0a, 0x06, 0x16, 0x0e, 0x1e, 0x24, 0x2c, 0x10, 0x30, 0x50, 0x70, 0x90, 0xb0, 0xd0,
        0xf0, 0x00, 0xc9, 0xc5, 0xd5, 0xcd, 0xdd, 0xd9, 0xc1, 0xd1, 0xe0, 0xe4, 0xec, 0xc0, 0xc4,
        0xcc, 0xc6, 0xd6, 0xce, 0xde, 0x49, 0x45, 0x55, 0x4d, 0x5d, 0x59, 0x41, 0x51, 0x18, 0x38,
        0x58, 0x78, 0xb8, 0xd8, 0xf8, 0xe6, 0xf6, 0xee, 0xfe, 0x4c, 0x6c, 0x20, 0xa9, 0xa5, 0xb5,
        0xad, 0xbd, 0xb9, 0xa1, 0xb1, 0xa2, 0xa6, 0xb6, 0xae, 0xbe, 0xa0, 0xa4, 0xb4, 0xac, 0xbc,
        0x4a, 0x46, 0x56, 0x4e, 0x5e, 0xea, 0x09, 0x05, 0x15, 0x0d, 0x1d, 0x19, 0x01, 0x11, 0xaa,
        0x8a, 0xca, 0xe8, 0xa8, 0x98, 0x88, 0xc8, 0x2a, 0x26, 0x36, 0x2e, 0x3e, 0x6a, 0x66, 0x76,
        0x6e, 0x7e, 0x40, 0x60, 0xe9, 0xe5, 0xf5, 0xed, 0xfd, 0xf9, 0xe1, 0xf1, 0x85, 0x95, 0x8d,
        0x9d, 0x99, 0x81, 0x91, 0x9a, 0xba, 0x48, 0x68, 0x08, 0x28, 0x86, 0x96, 0x8e, 0x84, 0x94,
        0x8c,
    ]
    .choose(&mut rand::thread_rng())
    .unwrap()
}

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

fn decode(machine_code: &[u8]) -> (Opcode, Operand) {
    let mut inst = YaxpeaxInstruction::default();
    let decoder = yaxpeax_6502::InstDecoder;
    let mut reader = U8Reader::new(machine_code);
    decoder.decode_into(&mut inst, &mut reader).unwrap();
    (inst.opcode, inst.operand)
}

/// Represents a 6502 Instruction
#[derive(Clone, Debug)]
pub struct Instruction6502 {
    opcode: u8,
    operand1: Option<u8>,
    operand2: Option<u8>,
}

impl Instruction6502 {
    /// returns true iff the instruction reads the X register
    pub fn reads_x(&self) -> bool {
        match decode(&self.to_bytes()) {
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
        match decode(&self.to_bytes()) {
            (Opcode::TAX, _) => true,
            (Opcode::LDX, _) => true,
            (_, _) => false,
        }
    }

    /// returns true iff the instruction is a conditional branch
    fn is_branch(&self) -> bool {
        match decode(&self.to_bytes()) {
            (Opcode::BCC, _) => true,
            (Opcode::BCS, _) => true,
            (Opcode::BEQ, _) => true,
            (Opcode::BMI, _) => true,
            (Opcode::BNE, _) => true,
            (Opcode::BPL, _) => true,
            (Opcode::BVC, _) => true,
            (Opcode::BVS, _) => true,
            (_, _) => false,
        }
    }

    /// returns true iff the instruction is a forward branch
    fn is_forward_branch(&self) -> bool {
        // If this unwrap panics, it's because generation of branch
        // instructions doesn't set self.operand1, as it should
        self.is_branch() && (self.operand1.unwrap() & 0x80 != 0x80)
    }

    /// returns true iff the instruction is a backward branch
    fn is_backward_branch(&self) -> bool {
        self.is_branch() && !self.is_forward_branch()
    }
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (opcode, operand) = decode(&self.to_bytes());
        let b1 = format!("${:02x}", self.opcode);
        let b2 = if let Some(v) = self.operand1 {
            format!("${:02x}", v)
        } else {
            "   ".to_string()
        };
        let b3 = if let Some(v) = self.operand2 {
            format!("${:02x}", v)
        } else {
            "   ".to_string()
        };

        let st = format!(
            "{} {} {}   {}",
            b1,
            b2,
            b3,
            format!("{}", opcode).to_lowercase()
        );

        match operand {
            Operand::Implied => write!(f, "{}", st),
            Operand::Accumulator => write!(f, "{} a", st),
            Operand::Immediate(val) => write!(f, "{} ${:02x}", st, val),
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
        match (self.operand1, self.operand2) {
            (None, None) => 1,
            (Some(_), None) => 2,
            (Some(_), Some(_)) => 3,
            (None, Some(_)) => panic!(),
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let rand: Vec<u8> = vec![random_codepoint(), random(), random()];
        let (_insn, operand) = decode(&rand);
        match instruction_length(operand) {
            1 => Instruction6502 {
                opcode: rand[0],
                operand1: None,
                operand2: None,
            },
            2 => Instruction6502 {
                opcode: rand[0],
                operand1: Some(rand[1]),
                operand2: None,
            },
            3 => Instruction6502 {
                opcode: rand[0],
                operand1: Some(rand[1]),
                operand2: Some(rand[2]),
            },
            _ => panic!(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match (self.operand1, self.operand2) {
            (None, None) => vec![self.opcode],
            (Some(op1), None) => vec![self.opcode, op1],
            (Some(op1), Some(op2)) => vec![self.opcode, op1, op2],
            (None, Some(_)) => panic!(),
        }
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        Box::new(self.to_bytes().into_iter())
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::mos6502::Instruction6502;

    #[test]
    fn new_instructions() {
        for _i in 0..50000 {
            let mut insn = Instruction6502::new();
        }
    }
}
