use crate::z80::Insn;
use crate::Peephole;

fn cp1(a: &[u8], a2: &[u8]) -> bool {
    let len = a2.len();
    if a.len() < len {
        return false;
    }
    a[..len] == a2[..len]
}

fn cp2(a: &[u8], b: &[u8], a2: &[u8], b2: &[u8]) -> bool {
    cp1(a, a2) && cp1(b, b2)
}

macro_rules! cp {
    // This macro takes an expression of type `expr` and prints
    // it as a string along with its result.
    // The `expr` designator is used for expressions.
    ($a:expr, $b:expr, $a2:expr, $b2:expr) => {
        if cp1($a, $a2) {
            if $b2.iter().any(|b2| !cp1($b, b2)) {
                return true;
            }
        } else {
            return false;
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

        // pointless instructions that load from one register to the same register, like `ld c, c,`
        for i in [0x40, 0x049, 0x52, 0x5b, 0x64, 0x6d, 0x7f] {
            if a_encoded[0] == i {
                return true;
            }
        }

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

        false
    }
}
