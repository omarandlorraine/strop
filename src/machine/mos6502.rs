//! The `mos6502` backend, for generating code sequences for the famous 8-bit
//! CPU from 1975. It also supports the later CMOS opcodes and known illegal opcodes
//! present on the NMOS models.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::machine::Instruction;
use mos6502::cpu::CPU;
use std::fmt::Debug;
use std::fmt::Formatter;
use yaxpeax_6502::Operand;

/// The internal state of a 6502
#[derive(Debug)]
pub struct Mos6502 {
    cpu: CPU,
}

impl Default for Mos6502 {
    fn default() -> Self {
        Mos6502 { cpu: CPU::new() }
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
        fn high(addr: u16) -> u8 {
            addr.to_le_bytes()[1]
        }

        fn low(addr: u16) -> u8 {
            addr.to_le_bytes()[0]
        }
        Box::new(
            match self.operand {
                Operand::Implied => vec!(self.opcode),
                Operand::Accumulator => vec!(self.opcode),
                Operand::Immediate(val) => vec!(self.opcode, val),
                Operand::IndirectYIndexed(addr) | Operand::XIndexedIndirect(addr) => {
                    vec!(self.opcode, addr)
                }
                Operand::ZeroPage(addr) | Operand::ZeroPageX(addr) | Operand::ZeroPageY(addr) => {
                    vec!(self.opcode, addr)
                }
                Operand::Relative(offset) => vec!(self.opcode, offset),
                Operand::Indirect(addr) => vec!(self.opcode, low(addr), high(addr)),
                Operand::Absolute(addr) | Operand::AbsoluteX(addr) | Operand::AbsoluteY(addr) => {
                    vec!(self.opcode, low(addr), high(addr))
                }
            }
            .into_iter())
    }
}
