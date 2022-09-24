//! The `mos6502` backend, for generating code sequences for the famous 8-bit
//! CPU from 1975. It also supports the later CMOS opcodes and known illegal opcodes
//! present on the NMOS models.

#![warn(missing_debug_implementations, missing_docs)]

use mos6502::cpu::CPU;
use yaxpeax_6502::Operand;
use crate::machine::Instruction;
use std::fmt::Debug;
use std::fmt::Formatter;

/// The internal state of a 6502
#[derive(Debug)]
pub struct Mos6502 {
    cpu: CPU,
}

impl Default for Mos6502 {
    fn default() -> Self {
        Mos6502 {
            cpu: CPU::new(),
        }
    }
}


impl Debug for Instruction6502 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

/// Represents a 6502 Instruction
#[derive(Clone)]
pub struct Instruction6502 {
    randomizer: fn(&mut Instruction6502),
    disassemble: fn(&Instruction6502, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut Mos6502),
    opcode: u8,
    operand: Operand,
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

struct ByteIterator6502 {
    opcode: u8,
    first: Option<u8>,
    second: Option<u8>,
    count: usize,
}

impl ByteIterator6502 {
    fn new(opcode: u8, operand: Operand) -> ByteIterator6502 {
        fn high(addr: u16) -> Option<u8> {
            Some(addr.to_le_bytes()[1])
        }

        fn low(addr: u16) -> Option<u8> {
            Some(addr.to_le_bytes()[0])
        }

        fn build(opcode: u8, first: Option<u8>, second: Option<u8>) -> ByteIterator6502 {
            ByteIterator6502 {
                opcode,
                first,
                second,
                count: 0,
            }
        }

        match operand {
            Operand::Implied => build(opcode, None, None),
            Operand::Accumulator => build(opcode, None, None),
            Operand::Immediate(val) => build(opcode, Some(val), None),
            Operand::IndirectYIndexed(addr) | Operand::XIndexedIndirect(addr) => build(opcode, Some(addr), None),
            Operand::ZeroPage(addr) | Operand::ZeroPageX(addr) | Operand::ZeroPageY(addr) => build(opcode, Some(addr), None),
            Operand::Relative(offset) => build(opcode, Some(offset), None),
            Operand::Indirect(addr) => build(opcode, low(addr), high(addr)),
            Operand::Absolute(addr) | Operand::AbsoluteX(addr) | Operand::AbsoluteY(addr) => build(opcode, low(addr), high(addr)),
        }
    }
}

impl Iterator for ByteIterator6502 {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.count += 1;
        match self.count {
            1 => Some(self.opcode),
            2 => self.first,
            3 => self.second,
            4 => None,
            _ => panic!(),
        }
    }
}

impl Instruction for Instruction6502 {
    type State = Mos6502;

    fn randomize(&mut self) {
        (self.randomizer)(self);
    }

    fn operate(&self, s: &mut Mos6502) {
        (self.handler)(self, s);
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
        panic!()
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        Box::new(ByteIterator6502::new(self.opcode, self.operand))
    }
}
