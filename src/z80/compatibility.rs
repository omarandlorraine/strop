use crate::z80::Z80Instruction;
use crate::Fixup;
use crate::Instruction;

fn opcode_present_on_8080(opcode: u8) -> bool {
    // excludes the opcodes:
    //  - `ex af,af'` and `exx`
    //  - the relative jumps
    //  - `djnz`
    !matches!(
        opcode,
        0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xd9 | 0xcb | 0xed | 0xdd | 0xfd
    )
}

fn opcode_present_on_sm83(opcode: u8, byte2: u8) -> bool {
    // this information is sourced from https://gbdev.io/pandocs/CPU_Comparison_with_Z80.html
    // excludes the opcodes:
    // 08	ex af,af
    // 10	djnz pc+dd
    // 22	ld (nn),hl
    // 2A	ld hl,(nn)
    // 32	ld (nn),a
    // 3A	ld a,(nn)
    // D3	out (n),a
    // D9	exx
    // DB	in a,(n)
    // DD	<IX> prefix
    // E0	ret po
    // E2	jp po,nn
    // E3	ex (sp),hl
    // E4	call p0,nn
    // E8	ret pe
    // EA	jp pe,nn
    // EB	ex de,hl
    // EC	call pe,nn
    // ED	<prefix>
    // F0	ret p
    // F2	jp p,nn
    // F4	call p,nn
    // F8	ret m
    // FA	jp m,nn
    // FC	call m,nn
    // FD	<IY> prefix
    // CB 3X	sll <something>
    match opcode {
        0x08 | 0x10 | 0x22 | 0x2a | 0x32 | 0x3a | 0xd3 | 0xd9 | 0xdb | 0xdd | 0xe0 | 0xe2
        | 0xe3 | 0xe4 | 0xe8 | 0xea | 0xeb | 0xec | 0xed | 0xf0 | 0xf2 | 0xf4 | 0xf8 | 0xfa
        | 0xfc | 0xfd => false,
        0xcb => byte2 & 0xf0 != 0x30,
        _ => true,
    }
}

/// Fixup ensuring that the Z80 instruction is present on the Intel 8080
#[derive(Debug)]
pub struct I8080Compatibility;

impl Fixup<Z80Instruction> for I8080Compatibility {
    fn random(&self, _insn: Z80Instruction) -> Z80Instruction {
        use crate::Instruction;
        loop {
            let insn = Z80Instruction::random();
            if self.check(insn) {
                return insn;
            }
        }
    }

    fn next(&self, insn: Z80Instruction) -> Option<Z80Instruction> {
        insn.increment_opcode()
    }

    fn check(&self, insn: Z80Instruction) -> bool {
        !opcode_present_on_8080(insn.encode()[0])
    }
}

/// Fixup ensuring that the Z80 instruction is present on the SM83 (aka. Gameboy)
#[derive(Debug)]
pub struct Sm83Compatibility;

impl Fixup<Z80Instruction> for Sm83Compatibility {
    fn random(&self, _insn: Z80Instruction) -> Z80Instruction {
        use crate::Instruction;
        loop {
            let insn = Z80Instruction::random();
            if self.check(insn) {
                return insn;
            }
        }
    }

    fn next(&self, insn: Z80Instruction) -> Option<Z80Instruction> {
        match insn.increment_opcode() {
            Some(i) => {
                if self.check(i) {
                    self.next(i)
                } else {
                    Some(i)
                }
            }
            None => None,
        }
    }

    fn check(&self, insn: Z80Instruction) -> bool {
        opcode_present_on_sm83(insn.encode()[0], insn.encode()[1])
    }
}
