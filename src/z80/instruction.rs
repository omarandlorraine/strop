//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::instruction::Instruction;
use crate::randomly;
use dez80::Instruction as DeZ80Instruction;
use rand::prelude::SliceRandom;
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
    fn encode(&mut self, insn: dez80::instruction::Instruction) {
        let istream: Vec<_> = insn.to_bytes();
        for i in 0..istream.len() {
            self.encoding[i] = istream[i];
        }
    }

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

    fn mutate_operand(&mut self) {
        use dez80::instruction::Operand::*;
        use dez80::RegisterPairType;
        use dez80::SingleRegisterType;

        let mut insn = self.decode();

        fn r() -> SingleRegisterType {
            use dez80::SingleRegisterType::*;
            *vec![A, F, B, C, D, E, H, L, IXH, IXL, IYH, IYL]
                .choose(&mut rand::thread_rng())
                .unwrap()
        }

        fn rp() -> RegisterPairType {
            use dez80::RegisterPairType::*;
            *vec![AF, BC, DE, HL, IX, IY, PC, SP]
                .choose(&mut rand::thread_rng())
                .unwrap()
        }

        let operand = *vec![
            OctetImmediate(random()),
            DoubletImmediate(random()),
            OctetImplied(random()),
            RegisterImplied(r()),
            RegisterPairImplied(rp()),
            RegisterImpliedBit(r(), random()),
            MemoryDirect(random()),
            MemoryIndirect(rp()),
            MemoryIndexed(rp(), random()),
            MemoryIndexedAndRegister(rp(), random(), r()),
            MemoryIndirectBit(rp(), random()),
            MemoryIndexedBit(rp(), random(), random()),
            MemoryIndexedBitAndRegister(rp(), random(), random(), r()),
            ProgramCounterRelative(random()),
            PortDirect(random()),
            PortIndirect(r()),
        ]
        .choose(&mut rand::thread_rng())
        .unwrap();

        randomly!({insn.source = Some(operand)} {insn.destination = Some(operand)});
    }

    fn mutate_opcode(&mut self) {
        use dez80::instruction::InstructionType::*;

        // TODO: Add conditional instructions here, like jr nz, something
        // TODO: Add im instructions here, im 0, im 1, im 2
        // TODO: Add the RSTs here

        let mut insn = self.decode();
        insn.r#type = *vec![
            Adc,
            Add,
            And,
            Bit,
            Call(None),
            Ccf,
            Cp,
            Cpd,
            Cpdr,
            Cpi,
            Cpir,
            Cpl,
            Daa,
            Dec,
            Di,
            Djnz,
            Ei,
            Ex,
            Exx,
            Halt,
            In,
            Inc,
            Ind,
            Indr,
            Ini,
            Inir,
            Jp(None),
            Jr(None),
            Ld,
            Ldd,
            Lddr,
            Ldi,
            Ldir,
            Neg,
            Nop,
            Or,
            Otdr,
            Otir,
            Out,
            Outd,
            Outi,
            Pop,
            Push,
            Res,
            Ret(None),
            Reti,
            Retn,
            Rl,
            Rla,
            Rlc,
            Rlca,
            Rld,
            Rr,
            Rra,
            Rrc,
            Rrca,
            Rrd,
            Sbc,
            Scf,
            Set,
            Sla,
            Sll,
            Sra,
            Srl,
            Sub,
            Xor,
        ]
        .choose(&mut rand::thread_rng())
        .unwrap();

        self.encode(insn);
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

            // Make sure the instruction can be disassembled (i. e., the disassembler doesn't bail
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
