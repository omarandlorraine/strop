//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![allow(dead_code)]

use crate::instruction::Instruction;
use rand::prelude::SliceRandom;
use rand::random;
use yaxpeax_6502::Instruction as YaxpeaxInstruction;
use yaxpeax_6502::{Opcode, Operand};
use yaxpeax_arch::Decoder;
use yaxpeax_arch::U8Reader;

use phf::{phf_set, Set};

pub const ACC_OPCODES: [u8; 4] = [0x4a, 0x6a, 0x2a, 0x0a];

pub const IMP_OPCODES: [u8; 25] = [
    0x60, 0xa8, 0xc8, 0x8a, 0x98, 0x08, 0x00, 0x68, 0xea, 0xca, 0xf8, 0x38, 0x40, 0x48, 0x18, 0xe8,
    0xd8, 0x58, 0x78, 0x28, 0xba, 0xaa, 0x88, 0xb8, 0x9a,
];

pub const IMM_OPCODES: [u8; 11] = [
    0xa2, 0xe9, 0xc9, 0x09, 0x49, 0x29, 0xe0, 0xa9, 0xc0, 0xa0, 0x69,
];

pub const ZP_OPCODES: [u8; 21] = [
    0x86, 0x24, 0xc6, 0x84, 0x66, 0xe5, 0xa6, 0x85, 0xa5, 0x65, 0x05, 0xc4, 0xe6, 0x06, 0x26, 0xa4,
    0x25, 0xe4, 0x46, 0xc5, 0x45,
];

pub const ZPX_OPCODES: [u8; 16] = [
    0xd6, 0xb4, 0x35, 0xd5, 0xb5, 0x76, 0x36, 0x95, 0xf6, 0x94, 0xf5, 0x16, 0x55, 0x75, 0x15, 0x56,
];

pub const ZPY_OPCODES: [u8; 2] = [0x96, 0xb6];

pub const ABS_OPCODES: [u8; 23] = [
    0x20, 0x8d, 0xed, 0x4e, 0xcd, 0xae, 0x2e, 0x6e, 0xce, 0xac, 0x8c, 0xcc, 0xec, 0x2c, 0x4c, 0x4d,
    0x2d, 0x0e, 0xee, 0x6d, 0x8e, 0x0d, 0xad,
];

pub const ABSX_OPCODES: [u8; 15] = [
    0x9d, 0xfe, 0x7e, 0x3e, 0x5d, 0xfd, 0x5e, 0x7d, 0xde, 0x3d, 0x1e, 0xbd, 0x1d, 0xdd, 0xbc,
];

pub const ABSY_OPCODES: [u8; 9] = [0xbe, 0x79, 0xf9, 0xb9, 0xd9, 0x39, 0x99, 0x19, 0x59];

pub const IND_OPCODES: [u8; 1] = [0x6c];

pub const INDX_OPCODES: [u8; 8] = [0x81, 0x41, 0xc1, 0x21, 0xe1, 0x61, 0xa1, 0x01];

pub const INDY_OPCODES: [u8; 8] = [0xd1, 0x11, 0xb1, 0x51, 0xf1, 0x31, 0x91, 0x71];

pub const REL_OPCODES: [u8; 8] = [0x70, 0x50, 0xf0, 0x30, 0xd0, 0x10, 0x90, 0xb0];

pub const ALL_OPCODES: [u8; 151] = [
    0x69, 0x65, 0x75, 0x6d, 0x7d, 0x79, 0x61, 0x71, 0x29, 0x25, 0x35, 0x2d, 0x3d, 0x39, 0x21, 0x31,
    0x0a, 0x06, 0x16, 0x0e, 0x1e, 0x24, 0x2c, 0x10, 0x30, 0x50, 0x70, 0x90, 0xb0, 0xd0, 0xf0, 0x00,
    0xc9, 0xc5, 0xd5, 0xcd, 0xdd, 0xd9, 0xc1, 0xd1, 0xe0, 0xe4, 0xec, 0xc0, 0xc4, 0xcc, 0xc6, 0xd6,
    0xce, 0xde, 0x49, 0x45, 0x55, 0x4d, 0x5d, 0x59, 0x41, 0x51, 0x18, 0x38, 0x58, 0x78, 0xb8, 0xd8,
    0xf8, 0xe6, 0xf6, 0xee, 0xfe, 0x4c, 0x6c, 0x20, 0xa9, 0xa5, 0xb5, 0xad, 0xbd, 0xb9, 0xa1, 0xb1,
    0xa2, 0xa6, 0xb6, 0xae, 0xbe, 0xa0, 0xa4, 0xb4, 0xac, 0xbc, 0x4a, 0x46, 0x56, 0x4e, 0x5e, 0xea,
    0x09, 0x05, 0x15, 0x0d, 0x1d, 0x19, 0x01, 0x11, 0xaa, 0x8a, 0xca, 0xe8, 0xa8, 0x98, 0x88, 0xc8,
    0x2a, 0x26, 0x36, 0x2e, 0x3e, 0x6a, 0x66, 0x76, 0x6e, 0x7e, 0x40, 0x60, 0xe9, 0xe5, 0xf5, 0xed,
    0xfd, 0xf9, 0xe1, 0xf1, 0x85, 0x95, 0x8d, 0x9d, 0x99, 0x81, 0x91, 0x9a, 0xba, 0x48, 0x68, 0x08,
    0x28, 0x86, 0x96, 0x8e, 0x84, 0x94, 0x8c,
];

fn random_codepoint() -> u8 {
    // returns one random, valid opcode
    *ALL_OPCODES.choose(&mut rand::thread_rng()).unwrap()
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::mos6502::data::ALL_OPCODES;
    use crate::mos6502::data::ABSX_OPCODES;
    use crate::mos6502::data::ABSY_OPCODES;
    use crate::mos6502::data::ABS_OPCODES;
    use crate::mos6502::data::ACC_OPCODES;
    use crate::mos6502::data::IMM_OPCODES;
    use crate::mos6502::data::IMP_OPCODES;
    use crate::mos6502::data::INDX_OPCODES;
    use crate::mos6502::data::INDY_OPCODES;
    use crate::mos6502::data::IND_OPCODES;
    use crate::mos6502::data::REL_OPCODES;
    use crate::mos6502::data::ZPX_OPCODES;
    use crate::mos6502::data::ZPY_OPCODES;
    use crate::mos6502::data::ZP_OPCODES;
    use crate::mos6502::Instruction6502;
    use yaxpeax_6502::{Opcode, Operand};

    #[test]
    fn uniq() {
        let mut found: Vec<u8> = vec![];

        fn check(op: u8, found: &mut Vec<u8>) {
            if found.contains(&op) {
                panic!()
            }
            found.append(&mut vec![op]);
        }

        for op in ABS_OPCODES { check(op, &mut found) }
        for op in ABSX_OPCODES { check(op, &mut found) }
        for op in ABSY_OPCODES { check(op, &mut found) }
        for op in ACC_OPCODES { check(op, &mut found) }
        for op in IMM_OPCODES { check(op, &mut found) }
        for op in IMP_OPCODES { check(op, &mut found) }
        for op in INDX_OPCODES { check(op, &mut found) }
        for op in INDY_OPCODES { check(op, &mut found) }
        for op in IND_OPCODES { check(op, &mut found) }
        for op in REL_OPCODES { check(op, &mut found) }
        for op in ZPX_OPCODES { check(op, &mut found) }
        for op in ZPY_OPCODES { check(op, &mut found) }
        for op in ZP_OPCODES { check(op, &mut found) }

        for op in ALL_OPCODES {
            if !found.contains(&op) {
                panic!();
            }
        }
    }
}
