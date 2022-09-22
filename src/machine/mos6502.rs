//! The `mos6502` backend, for generating code sequences for the famous 8-bit
//! CPU from 1975. It also supports the later CMOS opcodes and known illegal opcodes
//! present on the NMOS models.

#![warn(missing_debug_implementations, missing_docs)]

use crate::machine::Instruction;
use crate::randomly;
use rand::random;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;

/// The internal state of a 6502
#[derive(Debug, Default)]
pub struct Mos6502 {
    /// The A register
    pub a: Option<u8>,

    /// The X register
    pub x: Option<u8>,

    /// The Y register
    pub y: Option<u8>,

    /// Stack pointer
    pub s: u8,

    /// Memory
    pub heap: HashMap<u16, Option<u8>>,

    /// Carry flag
    pub carry: Option<bool>,

    /// Zero flag
    pub zero: Option<bool>,

    /// Sign flag
    pub sign: Option<bool>,

    /// Overflow flag
    pub overflow: Option<bool>,

    /// Decimal flag
    pub decimal: Option<bool>,

    /// True iff a CMOS-only instruction has been run. (You may want to use this flag in your cost
    /// function to determine if the program will run at all on your device)
    pub requires_cmos: bool,

    /// True iff an illegal instruction has been run. (You may want to use this flag in your cost
    /// function to determine if the program will run reliably on your device)
    pub illegal: bool,

    /// True iff a ROR instruction has been run. (Very early parts do not have this opcode; if you
    /// intend to use such a specimen, then you may want to use this flag in your cost function to
    /// determine if the program will run at all on your device)
    pub requires_ror: bool,
}

impl Mos6502 {
    fn read_mem(&self, addr: u16) -> Option<u8> {
        *self.heap.get(&addr).unwrap_or(&None)
    }

    fn write_mem(&mut self, addr: u16, val: Option<u8>) {
        self.heap.insert(addr, val);
    }

    fn push(&mut self, val: Option<u8>) {
        let addr: u16 = 0x0100 + self.s as u16;
        self.write_mem(addr, val);
        self.s = self.s.wrapping_sub(1);
    }

    fn pull(&mut self) -> Option<u8> {
        self.s = self.s.wrapping_add(1);
        let addr: u16 = 0x0100 + self.s as u16;
        self.read_mem(addr)
    }
}

/// A 6502 instruction's operand
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operand6502 {
    /// Used for implicit instructions, which take no operand
    None,

    /// For RMW instructions operating on the accumulator
    A,

    /// An immediate value
    Immediate(u8),

    /// Absolute addressing mode
    Absolute(u16),
}

impl Operand6502 {
    fn get(self, s: &Mos6502) -> Option<u8> {
        match self {
            Operand6502::None => panic!(),
            Operand6502::A => s.a,
            Operand6502::Immediate(v) => Some(v),
            Operand6502::Absolute(addr) => s.read_mem(addr),
        }
    }
    fn set(self, s: &mut Mos6502, val: Option<u8>) {
        match self {
            Operand6502::None => panic!(),
            Operand6502::A => s.a = val,
            Operand6502::Immediate(_) => panic!(),
            Operand6502::Absolute(addr) => s.write_mem(addr, val),
        }
    }
}

fn disassemble(insn: &Instruction6502, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match insn.operand {
        Operand6502::None => {
            write!(f, "\t{}", insn.mnem)
        }
        Operand6502::A => {
            write!(f, "\t{} a", insn.mnem)
        }
        Operand6502::Immediate(val) => {
            write!(f, "\t{} #${:02x}", insn.mnem, val)
        }
        Operand6502::Absolute(addr) => {
            write!(f, "\t{} ${:04x}", insn.mnem, addr)
        }
    }
}

impl Debug for Instruction6502 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

/// Represents a 6502 Instruction
#[derive(Clone, Copy)]
pub struct Instruction6502 {
    mnem: &'static str,
    randomizer: fn(&mut Instruction6502),
    disassemble: fn(&Instruction6502, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut Mos6502),
    opcode: u8,
    operand: Operand6502,
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
    count: usize
}

impl ByteIterator6502 {
    fn new(opcode: u8, operand: Operand6502) -> ByteIterator6502 {
        fn high(addr: u16) -> Option<u8> {
            Some(addr.to_le_bytes()[1])
        }

        fn low(addr: u16) -> Option<u8> {
            Some(addr.to_le_bytes()[0])
        }

        fn build(opcode: u8, first: Option<u8>, second: Option<u8>) -> ByteIterator6502 {
            ByteIterator6502 {
                opcode, first, second,
                count: 0
            }
        }

        match operand {
            Operand6502::None => build(opcode, None, None),
            Operand6502::A => build(opcode, None, None),
            Operand6502::Immediate(val) => build(opcode, Some(val), None),
            Operand6502::Absolute(addr) => build(opcode, low(addr), high(addr)),
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
            _ => panic!()
        }
    }
}

impl Instruction for Instruction6502 {
    type State = Mos6502;

    fn randomize(&mut self) {
        (self.randomizer)(self);
    }

    fn length(&self) -> usize {
        match self.operand {
            Operand6502::None => 1usize,
            Operand6502::A => 1usize,
            Operand6502::Immediate(_) => 2usize,
            Operand6502::Absolute(_) => 3usize,
        }
    }

    fn operate(&self, s: &mut Mos6502) {
        (self.handler)(self, s);
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
