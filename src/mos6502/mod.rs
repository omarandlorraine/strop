//! Module containing everything needed to use Strop to generate code for the MOS 6502

pub mod emulators;
pub mod instruction_set;

use crate::mos6502::instruction_set::Cmos6502Instruction;
use crate::mos6502::instruction_set::Nmos6502Instruction;
use crate::Compatibility;
use crate::SearchCull;

struct RevisionA;

fn nmos_skip_jump_bug(instruction: &Nmos6502Instruction) -> SearchCull<Nmos6502Instruction> {
    // indirect jumps can't dereference pointers straddling a page boundary on some 6502s, so
    // here's a convenience function to convert these to an appropriate SearchCull
    use crate::Instruction;
    let enc = instruction.encode();
    if (enc[0], enc[1]) == (0x6c, 0xff) {
        if enc[1] == 0xff {
            SearchCull::SkipTo(Some(Nmos6502Instruction::new([0x6d, 0, 0])))
        } else {
            SearchCull::SkipTo(Some(Nmos6502Instruction::new([0x6c, 0, enc[2] + 1])))
        }
    } else {
        SearchCull::Okay
    }
}

impl Compatibility<Nmos6502Instruction> for RevisionA {
    fn check(&self, instruction: &Nmos6502Instruction) -> SearchCull<Nmos6502Instruction> {
        use crate::Instruction;
        let enc = instruction.encode();

        if matches!(enc[0], 0x6a | 0x6b | 0x76 | 0x6e | 0x7e) {
            SearchCull::SkipTo(Some(Nmos6502Instruction::new([0x6b, 0, 0])))
        } else {
            nmos_skip_jump_bug(instruction)
        }
    }
}

/// A type representing some "average 6502". A pretty safe bet. It rejects all CMOS-only
/// instructions, all NMOS "illegal opcodes", instructions exhibiting the jump indirect bug, and
/// instructions setting or clearing the decimal mode.
struct Average6502;

fn average_6502_check(opcode: u8) -> Option<u8> {
    let mut next_opcode = opcode;
    while !matches!(next_opcode, 0x69 | 0x6d | 0x7d | 0x79 | 0x65 | 0x75 | 0x61 | 0x71 | 0x29 | 0x2d | 0x3d | 0x39 | 0x25 | 0x35 | 0x21 | 0x31 | 0x0a | 0x0e | 0x1e | 0x06 | 0x16 | 0x90 | 0xb0 | 0xf0 | 0x2c | 0x24 | 0x30 | 0xd0 | 0x10 | 0x00 | 0x50 | 0x70 | 0x18 | 0x58 | 0xb8 | 0xc9 | 0xcd | 0xdd | 0xd9 | 0xc5 | 0xd5 | 0xc1 | 0xd1 | 0xe0 | 0xec | 0xe4 | 0xc0 | 0xcc | 0xc4 | 0xce | 0xde | 0xc6 | 0xd6 | 0xca | 0x88 | 0x49 | 0x4d | 0x5d | 0x59 | 0x45 | 0x55 | 0x41 | 0x51 | 0xee | 0xfe | 0xe6 | 0xf6 | 0xe8 | 0xc8 | 0x4c | 0x6c | 0x20 | 0xa9 | 0xad | 0xbd | 0xb9 | 0xa5 | 0xb5 | 0xa1 | 0xb1 | 0xa2 | 0xae | 0xbe | 0xa6 | 0xb6 | 0xa0 | 0xac | 0xbc | 0xa4 | 0xb4 | 0x4a | 0x4e | 0x5e | 0x46 | 0x56 | 0xea | 0x09 | 0x0d | 0x1d | 0x19 | 0x05 | 0x15 | 0x01 | 0x11 | 0x48 | 0x08 | 0x68 | 0x28 | 0x2a | 0x2e | 0x3e | 0x26 | 0x36 | 0x6a | 0x6e | 0x7e | 0x66 | 0x76 | 0x40 | 0x60 | 0xe9 | 0xed | 0xfd | 0xf9 | 0xe5 | 0xf5 | 0xe1 | 0xf1 | 0x38 | 0x78 | 0x8d | 0x9d | 0x99 | 0x85 | 0x95 | 0x81 | 0x91 | 0x8e | 0x86 | 0x96 | 0x8c | 0x84 | 0x94 | 0xaa | 0xa8 | 0xba | 0x8a | 0x9a | 0x98) {
        next_opcode += 1;
    }

    if opcode == next_opcode {
        // the opcode we've got is actually okay
        None
    } else {
        Some(next_opcode)
    }
}

impl Compatibility<Nmos6502Instruction> for Average6502 {
    fn check(&self, instruction: &Nmos6502Instruction) -> SearchCull<Nmos6502Instruction> {
        use crate::Instruction;
        let enc = instruction.encode();
        if enc[0] == 0xff {
            SearchCull::SkipTo(None)
        } else if let Some(o) = average_6502_check(enc[0]) {
            SearchCull::SkipTo(Some(Nmos6502Instruction::new([o, 0, 0])))
        } else {
            SearchCull::Okay
        }
    }
}

impl Compatibility<Cmos6502Instruction> for Average6502 {
    fn check(&self, instruction: &Cmos6502Instruction) -> SearchCull<Cmos6502Instruction> {
        use crate::Instruction;
        let enc = instruction.encode();
        if enc[0] == 0xff {
            SearchCull::SkipTo(None)
        } else if let Some(o) = average_6502_check(enc[0]) {
            SearchCull::SkipTo(Some(Cmos6502Instruction::new([o, 0, 0])))
        } else {
            SearchCull::Okay
        }
    }
}
