//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::instruction::Instruction;
use dez80::Instruction as DeZ80Instruction;
use rand::random;

//    DeZ80Instruction::decode_one(&mut data)

/// Represents a 6502 Instruction
#[derive(Clone, Debug)]
pub struct InstructionZ80 {
    encoding: [u8; 5],
}

impl std::fmt::Display for InstructionZ80 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let instr = DeZ80Instruction::decode_one(&mut self.encoding.as_slice()).unwrap();
        write!(f, "{}", instr)
    }
}

impl InstructionZ80 {
    fn decode(&self) -> DeZ80Instruction {
        DeZ80Instruction::decode_one(&mut self.encoding.as_slice()).unwrap()
    }

    fn compatible_with_sm83(&self) -> bool {
        //! Returns true if the instruction can be expected to run on the gameboy
        if !self.compatible_with_8080() {
            return false;
        }

        let bytes = self.to_bytes();
        let (opcode, prefixed) = (bytes[0], bytes[1]);

        if opcode == 0xcb {
            // On the Z80 this is the (illegal) SL1 instruction, but on the SM83 it's some kind of
            // SWAP instruction
            return prefixed & 0xf0 != 0x30;
        }

        return match opcode {
            //                     SM83 instruction   Z80 instruction
            0xf2 => false, //      ld   a,(c)         jp  p,nn
            0xe2 => false, //      ld   (c),a         jp  nv,nn
            0xea => false, //      ld   (nn),a        jp  v,nn
            0xfa => false, //      ld   a,(nn)        jp  m,nn
            0x3a => false, //      ldd  a,(hl)        ld  a,(nn)
            0x32 => false, //      ldd  (hl),a        ld  (nn),a
            0x2a => false, //      ldi  a,(hl)        ld  hl,(nn)
            0x22 => false, //      ldi  (hl),a        ld  (nn),hL
            0x08 => false, //      ld   (nn),sp       ex  af,af'
            0xe0 => false, //      ldh  (n),a         ret nv
            0xf0 => false, //      ldh  a,(n)         ret P
            0xf8 => false, //      ld   hl,(sp+e)     ret m
            0xe8 => false, //      add  sp,e          ret v
            0x10 => false, //      stop               djnz
            0xd9 => false, //      reti               exx
            _ => true,
        };
    }

    fn compatible_with_8080(&self) -> bool {
        //! Returns true if the instruction can be expected to run on the 8080
        //! (This includes instructions like `ADD`, which has a different effect on the Parity flag
        //! on the Z80 and on the 8080, so take care.)
        let opcode = self.to_bytes()[0];

        if opcode & 0x1c != 0x00 {
            // these opcodes are aliased to NOP on the 8080, but encode various new instructions on
            // the Z80. EX AF, AF', DJNZ, relative and conditional JR.
            return false;
        }

        if opcode == 0xcb {
            // this opcode is an instruction prefix on the Z80 but aliased to absolute JMP on the
            // 8080
            return false;
        }

        if opcode == 0xd9 {
            // EXX on the Z80
            return false;
        }

        if opcode == 0xdd {
            // IX prefix on the Z80
            return false;
        }

        if opcode == 0xed {
            // ED prefix for miscellaneous Z80 instructions
            return false;
        }

        if opcode == 0xfd {
            // ED prefix for miscellaneous Z80 instructions
            return false;
        }

        return true;
    }
}

impl Instruction for InstructionZ80 {
    fn length(&self) -> usize {
        let instr = DeZ80Instruction::decode_one(&mut self.encoding.as_slice()).unwrap();
        instr.to_bytes().len()
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        // Known issue: because of a bug in the dez80 crate, this does not generate any ED-prefixed
        // opcodes
        loop {
            let encoding: [u8; 5] = [random(), random(), random(), random(), random()];
            if encoding[0] != 0xed
                && ((encoding[0], encoding[1]) != (0xdd, 0xed))
                && DeZ80Instruction::decode_one(&mut encoding.as_slice()).is_ok()
            {
                return Self { encoding };
            }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let instr = DeZ80Instruction::decode_one(&mut self.encoding.as_slice()).unwrap();
        instr.to_bytes().to_vec()
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        Box::new(self.to_bytes().into_iter())
    }

    fn perm_bb(&self) -> bool {
        use dez80::instruction::InstructionType::*;
        match self.decode().r#type {
            Call(_) => false,
            Djnz => false,
            Halt => false,
            Jp(_) => false,
            Jr(_) => false,
            Ret(_) => false,
            Reti => false,
            Retn => false,
            Rst(_) => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::z80::instruction::DeZ80Instruction;
    use crate::z80::InstructionZ80;

    #[test]
    fn new_instructions() {
        for _i in 0..50000 {
            let insn = InstructionZ80::new();

            // Make sure the instruction can be disassembled (i. e., the diassembler doesn't bail
            // out and comment out the hex)
            let disasm = format!("{}", insn);
            assert!(
                disasm.chars().next().unwrap() != ';',
                "generated {} which has no encoding",
                disasm
            )
        }
    }

    #[test]
    fn illegal_instructions() {
        // Invalid instruction; semantically equivalent to NOP NOP
        assert!(DeZ80Instruction::decode_one(&mut vec![0xed, 0x0e].as_slice()).is_err())
    }
}
