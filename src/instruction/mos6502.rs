//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::instruction::Instruction;
use rand::prelude::SliceRandom;
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

fn decode(machine_code: Vec<u8>) -> (Opcode, Operand) {
    let mut inst = YaxpeaxInstruction::default();
    let decoder = yaxpeax_6502::InstDecoder;
    let mut reader = U8Reader::new(&machine_code);
    decoder.decode_into(&mut inst, &mut reader).unwrap();
    (inst.opcode, inst.operand)
}

/// Represents a 6502 Instruction
#[derive(Clone, Debug)]
pub struct Instruction6502 {
    opcode: u8,
    operand1: Option<u8>,
    operand2: Option<u8>,
    instruction: YaxpeaxInstruction,
    operand: Operand,
}

impl Instruction for Instruction6502 {
    fn randomize(&mut self) {
        panic!()
    }

    fn length(&self) -> usize {
        match self.operand {
            Operand::Implied => 1,
            Operand::Accumulator => 1,
            Operand::IndirectYIndexed(_) => 2,
            Operand::XIndexedIndirect(_) => 2,
            Operand::Relative(_) => 2,
            Operand::Indirect(_) => 3,
            Operand::Immediate(_) => 2,
            Operand::Absolute(_) => 3,
            Operand::AbsoluteX(_) => 3,
            Operand::AbsoluteY(_) => 3,
            Operand::ZeroPage(_) | Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => 2,
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        panic!();
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        fn high(addr: u16) -> u8 {
            addr.to_le_bytes()[1]
        }

        fn low(addr: u16) -> u8 {
            addr.to_le_bytes()[0]
        }
        Box::new(
            match self.operand {
                Operand::Implied => vec![self.opcode],
                Operand::Accumulator => vec![self.opcode],
                Operand::Immediate(val) => vec![self.opcode, val],
                Operand::IndirectYIndexed(addr) | Operand::XIndexedIndirect(addr) => {
                    vec![self.opcode, addr]
                }
                Operand::ZeroPage(addr) | Operand::ZeroPageX(addr) | Operand::ZeroPageY(addr) => {
                    vec![self.opcode, addr]
                }
                Operand::Relative(offset) => vec![self.opcode, offset],
                Operand::Indirect(addr) => vec![self.opcode, low(addr), high(addr)],
                Operand::Absolute(addr) | Operand::AbsoluteX(addr) | Operand::AbsoluteY(addr) => {
                    vec![self.opcode, low(addr), high(addr)]
                }
            }
            .into_iter(),
        )
    }
}
