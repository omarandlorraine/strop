#![allow(missing_docs)]

#[derive(Default, Debug)]
pub enum ReadWrite {
    /// Leaves the datum alone
    #[default]
    N,
    /// Read-modify-write
    Rmw,
    /// Read
    R,
    ///Write,
    W,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InstructionData {
    pub mnemonic: &'static str,
    pub opcode: u8,
    pub bytes: usize,
    pub cycles: usize,
    pub zero: ReadWrite,
    pub negative: ReadWrite,
    pub half_carry: ReadWrite,
    pub carry: ReadWrite,
    pub a: ReadWrite,
    pub b: ReadWrite,
    pub c: ReadWrite,
    pub d: ReadWrite,
    pub e: ReadWrite,
    pub h: ReadWrite,
    pub l: ReadWrite,
    pub iyl: ReadWrite,
    pub iyh: ReadWrite,
    pub ixl: ReadWrite,
    pub ixh: ReadWrite,
    pub r: ReadWrite,
    pub i: ReadWrite,
    pub sp: ReadWrite,

    pub operands: [&'static str; 3],
}
