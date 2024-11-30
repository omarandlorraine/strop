use crate::peephole::Peephole;
use crate::z80::Insn;

fn cp1(a: &[u8], a2: &[u8]) -> bool {
    let len = a2.len();
    if a.len() < len {
        return false;
    }
    a[..len] == a2[..len]
}

macro_rules! cp {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ($a:expr, $b:expr, $a2:expr, $b2:expr) => {
        if cp1($a, $a2) && $b2.iter().any(|b2| cp1($b, b2)) {
            return true;
        }
    };
}

impl Peephole for Insn {
    fn modify(&mut self) -> bool {
        self.next_opcode()
    }

    fn check(a: &Self, b: &Self) -> bool {
        use crate::Encode;

        let b_encoded = b.encode();

        // the function needs to return true if it can prove that the sequence [a, b] can be
        // improved on.
        if b_encoded[0] == 0x00 {
            // The second instruction is a no-op
            return true;
        }

        let a_encoded = a.encode();

        // pointless instructions that don't do anything, like `nop`, `ld c, c,`, etc.
        for i in [0x00, 0x40, 0x049, 0x52, 0x5b, 0x64, 0x6d, 0x7f] {
            if a_encoded[0] == i {
                return true;
            }
        }

        // pointless sequences that load one half of a register pair and then the other half. (for
        // example, there is no need for an `ld b, something` and then `ld c, something`, since
        // this sequence can be replaced by `ld bc, something`.
        cp!(&a_encoded, &b_encoded, &[0x01], &[[0x06], [0x0e]]);

        // pointless sequences that immediate load into A, and then performs some ALU operation on
        // it (a real compiler would just constant fold these two instructions together)
        cp!(
            &a_encoded,
            &b_encoded,
            &[0x3e],
            &[[0x3c], [0x3d], [0x3e], [0x87], [0xc6], [0xce]]
        );

        // pointless sequences that perform some operation on A, and then overwrite it
        cp!(
            &b_encoded,
            &a_encoded,
            &[0x3e],
            &[
                [0x3d],
                [0x3c],
                [0x3e],
                [0x87],
                [0x8f],
                [0x9f],
                [0xc6],
                [0xce],
                [0xde]
            ]
        );

        // No need for two consecutive immediate add/sub instructions
        let sub_and_add = [[0xc6], [0xd6], [0xce], [0xde]];
        for op in sub_and_add {
            cp!(&a_encoded, &b_encoded, &op, &sub_and_add);
        }

        // There's no need for an unconditional return instruction to be preceded by another return
        // instruction
        let return_instructions = [[0xc0, 0xd0, 0xe0, 0xf0, 0xc8, 0xd8, 0xe8, 0xf8]];
        cp!(&a_encoded, &b_encoded, &[0xc9], &return_instructions);
        cp!(&b_encoded, &a_encoded, &[0xc9], &return_instructions);

        // There's no need to load into BC and then increment, decrement or load BC or B or C,
        // Same deal with DE and HL
        for opcodes in [
            [0x01, 0x03, 0x04, 0x05, 0x06, 0x0b, 0x0c, 0x0d, 0x0e],
            [0x11, 0x13, 0x14, 0x15, 0x16, 0x1b, 0x1c, 0x1d, 0x1e],
            [0x21, 0x23, 0x24, 0x25, 0x26, 0x2b, 0x2c, 0x2d, 0x1e],
        ] {
            if opcodes.contains(&a_encoded[0]) && opcodes.contains(&b_encoded[0]) {
                return true;
            }
        }

        // instructions which could be scheduled either which way (that is, they are independent of
        // each other), but which are not in ascending numerical order.
        cp!(&a_encoded, &b_encoded, &[0x3e], &[[0x33, 0x3b]]);

        false
    }
}
