//! A module for the representation of SM83 machine instructions.

use crate::IterationResult;

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Debug, Default)]
pub struct Insn([u8; 3]);

impl crate::Step for Insn {
    fn first() -> Self {
        Self([0, 0, 0])
    }

    fn next(&mut self) -> IterationResult {
        if self.0[0] == 0xff {
            Err(crate::StepError::End)
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup();
            Ok(())
        }
    }
}

impl Insn {
    fn len(&self) -> usize {
        crate::i80::parse_sm83(&self.0).len
    }

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
}
