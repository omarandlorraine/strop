//! A module for the representation of SM83 machine instructions.

use crate::{IterationResult, StepError};

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 3]);

impl crate::Step for Insn {
    fn first() -> Self {
        Self([0, 0, 0])
    }

    fn next(&mut self) -> IterationResult {
        use crate::Encode;
        let len = self.len();
        if self.0[0] == 0xff {
            Err(StepError::End)
        } else {
            self.incr_at_offset(len - 1);
            self.fixup();
            Ok(())
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn len(&self) -> usize {
        self.decode().as_ref().map(|data| data.bytes).unwrap() as usize
    }
    
    fn encode(&self) -> Vec<u8> {
        let mut encoding = self.0.to_vec();
        encoding.truncate(self.len());
        encoding
    }
}

impl Insn {
    fn fixup(&mut self) {
        while matches!(
            self.0[0],
            0xd3 | 0xe3 | 0xe4 | 0xf4 | 0xdb | 0xeb | 0xec | 0xfc | 0xdd | 0xed | 0xfd
        ) {
            // illegal opcodes.
            self.0 = [self.0[0] + 1, 0, 0];
        }
    }

    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
    }

    fn decode(&self) -> &'static Option<InstructionData> {
        if self.0[0] == 0xcb {
            return &crate::sm83::data::CBPREFIXED[self.0[1] as usize];
        }
        return &crate::sm83::data::UNPREFIXED[self.0[0] as usize];
    }
}

#[derive(Default, Debug)]
pub(crate) enum ReadWrite {
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
pub(crate) struct InstructionData {
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
}
