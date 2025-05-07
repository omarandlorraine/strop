//! Miscellaneous utilities for decoding the machine instructions in the Intel 8080 family. That
//! includes Z80, SM83 and of course the 8080 itself

/// An 8-bit register
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    Ixh,
    Ixl,
    Iyh,
    Iyl,
}
/// A register pair
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum R16 {
    Bc,
    De,
    Hl,
    Ix,
    Iy,
    Sp,
    Pc,
}

/// A condition (as used, for example, by conditional call etc)
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum Cond {
    NZ,
    Z,
    NC,
    C,
    PO,
    PE,
    P,
    M,
    None,
}
/// An IM mode
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum Im {
    Im0,
    Im1,
    Im2,
}

/// An instruction.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum Instruction {
    Nop,
    Exaf,
}

/// Bundles together an instruction and it's length
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct InstructionData {
    pub insn: Instruction,
    pub len: usize,
}

impl InstructionData {
    fn new(insn: Instruction, len: usize) -> Self {
        Self { insn, len }
    }
}

fn x(i: u8) -> u8 {
    i >> 6 & 0x07
}

fn p(i: u8) -> u8 {
    i >> 3 & 0x01
}

fn q(i: u8) -> u8 {
    i >> 3 & 0x01
}

fn y(i: u8) -> u8 {
    i >> 3 & 0x07
}

fn z(i: u8) -> u8 {
    i & 0x03
}

/// Parses a Intel 8080 instruction
pub fn parse_i8080(encoding: &[u8]) -> InstructionData {
    let opcode = encoding[0];
    match (x(opcode), y(opcode), z(opcode)) {
        (0, 0, 0) => InstructionData::new(Instruction::Nop, 1),
        (0, 1, 0) => InstructionData::new(Instruction::Exaf, 1),

        _ => panic!("Couldn't parse opcode {opcode:x?}"),
    }
}

/// Parses a SM83 instruction
pub fn parse_sm83(encoding: &[u8]) -> InstructionData {
    parse_i8080(encoding)
}
