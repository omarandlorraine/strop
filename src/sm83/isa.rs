//! A module for the representation of SM83 machine instructions.

/// Represents a SM83 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 3]);

impl crate::Iterable for Insn {
    fn first() -> Self {
        Self([0, 0, 0])
    }

    fn step(&mut self) -> bool {
        use crate::Encode;
        if self.0[0] == 0xff {
            false
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup();
            true
        }
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
}
