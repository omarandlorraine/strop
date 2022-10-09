//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![allow(dead_code)]

use crate::instruction::Instruction;
use crate::randomly;
use rand::prelude::SliceRandom;
use rand::random;
use yaxpeax_6502::Instruction as YaxpeaxInstruction;
use yaxpeax_6502::{Opcode, Operand};
use yaxpeax_arch::Decoder;
use yaxpeax_arch::U8Reader;

fn random_codepoint() -> u8 {
    // returns one random, valid opcode
    *crate::mos6502::data::ALL_OPCODES
        .choose(&mut rand::thread_rng())
        .unwrap()
}

fn instruction_length(op: Operand) -> usize {
    match op {
        Operand::Accumulator | Operand::Implied => 1,
        Operand::Relative(_) => 2,
        Operand::Immediate(_) => 2,
        Operand::IndirectYIndexed(_) | Operand::XIndexedIndirect(_) => 2,
        Operand::ZeroPage(_) | Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => 2,
        Operand::Absolute(_) | Operand::AbsoluteX(_) | Operand::AbsoluteY(_) => 3,
        Operand::Indirect(_) => 3,
    }
}

pub fn decode(machine_code: &[u8]) -> (Opcode, Operand) {
    let mut inst = YaxpeaxInstruction::default();
    let decoder = yaxpeax_6502::InstDecoder;
    let mut reader = U8Reader::new(machine_code);
    decoder.decode_into(&mut inst, &mut reader).unwrap();
    (inst.opcode, inst.operand)
}

/// Represents a 6502 Instruction
#[derive(Clone, Copy, Debug)]
pub struct Instruction6502 {
    opcode: u8,
    operand1: Option<u8>,
    operand2: Option<u8>,
}

impl Instruction6502 {
    /// returns true iff the instruction reads the X register
    pub fn reads_x(&self) -> bool {
        match decode(&self.to_bytes()) {
            (_, Operand::XIndexedIndirect(_)) => true,
            (_, Operand::ZeroPageX(_)) => true,
            (_, Operand::AbsoluteX(_)) => true,
            (Opcode::TXA, _) => true,
            (Opcode::STX, _) => true,
            (Opcode::CPX, _) => true,
            (Opcode::DEX, _) => true,
            (Opcode::INX, _) => true,
            (Opcode::TXS, _) => true,
            (_, _) => false,
        }
    }

    /// returns true if the instruction sets the X register
    pub fn sets_x(&self) -> bool {
        match decode(&self.to_bytes()) {
            (Opcode::TAX, _) => true,
            (Opcode::LDX, _) => true,
            (_, _) => false,
        }
    }

    /// returns true iff the instruction is a conditional branch
    fn is_branch(&self) -> bool {
        match decode(&self.to_bytes()) {
            (Opcode::BCC, _) => true,
            (Opcode::BCS, _) => true,
            (Opcode::BEQ, _) => true,
            (Opcode::BMI, _) => true,
            (Opcode::BNE, _) => true,
            (Opcode::BPL, _) => true,
            (Opcode::BVC, _) => true,
            (Opcode::BVS, _) => true,
            (_, _) => false,
        }
    }

    /// returns true iff the instruction is a forward branch
    fn is_forward_branch(&self) -> bool {
        // If this unwrap panics, it's because generation of branch
        // instructions doesn't set self.operand1, as it should
        self.is_branch() && (self.operand1.unwrap() & 0x80 != 0x80)
    }

    /// returns true iff the instruction is a backward branch
    fn is_backward_branch(&self) -> bool {
        self.is_branch() && !self.is_forward_branch()
    }

    pub fn avoid_rorbug(&self) -> bool {
        //! Returns false if the instruction exercises the ROR bug.
        //! Early revisions of the MOS 6502 do not have this instruction
        !matches!(decode(&self.to_bytes()).0, Opcode::ROR)
    }
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (opcode, operand) = decode(&self.to_bytes());
        let b1 = format!("${:02x}", self.opcode);
        let b2 = if let Some(v) = self.operand1 {
            format!("${:02x}", v)
        } else {
            "   ".to_string()
        };
        let b3 = if let Some(v) = self.operand2 {
            format!("${:02x}", v)
        } else {
            "   ".to_string()
        };

        let st = format!(
            "{} {} {}   {}",
            b1,
            b2,
            b3,
            format!("{}", opcode).to_lowercase()
        );

        match operand {
            Operand::Implied => write!(f, "{}", st),
            Operand::Accumulator => write!(f, "{} a", st),
            Operand::Immediate(val) => write!(f, "{} ${:02x}", st, val),
            Operand::ZeroPage(addr) => write!(f, "{} ${:02x}", st, addr),
            Operand::ZeroPageX(addr) => write!(f, "{} ${:02x},x", st, addr),
            Operand::ZeroPageY(addr) => write!(f, "{} ${:02x},y", st, addr),
            Operand::Absolute(addr) => write!(f, "{} ${:04x}", st, addr),
            Operand::AbsoluteX(addr) => write!(f, "{} ${:04x},x", st, addr),
            Operand::AbsoluteY(addr) => write!(f, "{} ${:04x},y", st, addr),
            Operand::Indirect(addr) => write!(f, "{} (${:04x})", st, addr),
            Operand::IndirectYIndexed(addr) => write!(f, "{} (${:02x}),y", st, addr),
            Operand::XIndexedIndirect(addr) => write!(f, "{} (${:02x},x)", st, addr),
            Operand::Relative(offs) => write!(f, "{} ${:02x}", st, offs), // todo
        }
    }
}

impl Instruction for Instruction6502 {
    fn length(&self) -> usize {
        match (self.operand1, self.operand2) {
            (None, None) => 1,
            (Some(_), None) => 2,
            (Some(_), Some(_)) => 3,
            (None, Some(_)) => panic!(),
        }
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let rand: Vec<u8> = vec![random_codepoint(), random(), random()];
        let (_insn, operand) = decode(&rand);
        match instruction_length(operand) {
            1 => Instruction6502 {
                opcode: rand[0],
                operand1: None,
                operand2: None,
            },
            2 => Instruction6502 {
                opcode: rand[0],
                operand1: Some(rand[1]),
                operand2: None,
            },
            3 => Instruction6502 {
                opcode: rand[0],
                operand1: Some(rand[1]),
                operand2: Some(rand[2]),
            },
            _ => panic!(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match (self.operand1, self.operand2) {
            (None, None) => vec![self.opcode],
            (Some(op1), None) => vec![self.opcode, op1],
            (Some(op1), Some(op2)) => vec![self.opcode, op1, op2],
            (None, Some(_)) => panic!(),
        }
    }

    fn as_bytes(&self) -> Box<(dyn Iterator<Item = u8> + 'static)> {
        Box::new(self.to_bytes().into_iter())
    }

    fn perm_bb(&self) -> bool {
        !matches!(
            decode(&self.to_bytes()).0,
            Opcode::BCC
                | Opcode::BCS
                | Opcode::BEQ
                | Opcode::BMI
                | Opcode::BNE
                | Opcode::BPL
                | Opcode::BVC
                | Opcode::BVS
                | Opcode::JMP
                | Opcode::JSR
                | Opcode::RTS
                | Opcode::RTI
        )
    }

    fn mutate_operand(&mut self) {
        // Pick another opcode having the same addressing mode
        match decode(&self.to_bytes()).1 {
            Operand::Implied | Operand::Accumulator => (),
            Operand::Immediate(_) => {
                // try incrementing the value, decrementing it, or flipping a random bit
                randomly!(
                    { self.operand1 = self.operand1.map(|v| v.wrapping_add(1)) }
                    { self.operand1 = self.operand1.map(|v| v.wrapping_sub(1)) }
                    { let random_bit = *vec![0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80].choose(&mut rand::thread_rng()).unwrap();
                      self.operand1 = self.operand1.map(|v| v ^ random_bit);
                    }
                );
            }
            Operand::ZeroPage(_) | Operand::ZeroPageX(_) | Operand::ZeroPageY(_) => {
                self.operand1 = random();
            }
            Operand::Absolute(_)
            | Operand::AbsoluteX(_)
            | Operand::AbsoluteY(_)
            | Operand::Indirect(_) => {
                self.operand1 = random();
                self.operand2 = random();
            }
            Operand::IndirectYIndexed(_) | Operand::XIndexedIndirect(_) => {
                self.operand1 = random();
            }
            Operand::Relative(_) => {
                self.operand1 = random();
            }
        }
    }

    fn mutate_opcode(&mut self) {
        // Pick another opcode having the same addressing mode
        use crate::mos6502::data::*;

        self.opcode = match decode(&self.to_bytes()).1 {
            Operand::Implied => *IMP_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Accumulator => *ACC_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Immediate(_) => *IMM_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPage(_) => *ZP_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPageX(_) => *ZPX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::ZeroPageY(_) => *ZPY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Absolute(_) => *ABS_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::AbsoluteX(_) => *ABSX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::AbsoluteY(_) => *ABSY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Indirect(_) => *IND_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::IndirectYIndexed(_) => *INDX_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::XIndexedIndirect(_) => *INDY_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
            Operand::Relative(_) => *REL_OPCODES.choose(&mut rand::thread_rng()).unwrap(),
        };
    }
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
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
    use crate::mos6502::instruction::decode;
    use crate::mos6502::Instruction6502;
    use yaxpeax_6502::{Opcode, Operand};

    #[test]
    fn new_instructions() {
        for _i in 0..50000 {
            let insn = Instruction6502::new();

            insn.length();
            insn.as_bytes();

            let _disasm = format!("{}", insn);

            let opcode = insn.opcode;

            match decode(&insn.to_bytes()) {
                (_, Operand::Implied) => assert!(
                    IMP_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in IMP_OPCODES",
                    opcode
                ),
                (_, Operand::Accumulator) => assert!(
                    ACC_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ACC_OPCODES",
                    opcode
                ),
                (_, Operand::Immediate(_)) => assert!(
                    IMM_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in IMM_OPCODES",
                    opcode
                ),
                (_, Operand::Absolute(_)) => assert!(
                    ABS_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ABS_OPCODES",
                    opcode
                ),
                (_, Operand::AbsoluteX(_)) => assert!(
                    ABSX_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ABSX_OPCODES",
                    opcode
                ),
                (_, Operand::AbsoluteY(_)) => assert!(
                    ABSY_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ABSY_OPCODES",
                    opcode
                ),
                (_, Operand::Indirect(_)) => assert!(
                    IND_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in IND_OPCODES",
                    opcode
                ),
                (_, Operand::XIndexedIndirect(_)) => assert!(
                    INDX_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in INDX_OPCODES",
                    opcode
                ),
                (_, Operand::IndirectYIndexed(_)) => assert!(
                    INDY_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in INDY_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPage(_)) => assert!(
                    ZP_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ZP_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPageX(_)) => assert!(
                    ZPX_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ZPX_OPCODES",
                    opcode
                ),
                (_, Operand::ZeroPageY(_)) => assert!(
                    ZPY_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in ZPY_OPCODES",
                    opcode
                ),
                (_, Operand::Relative(_)) => assert!(
                    REL_OPCODES.contains(&insn.opcode),
                    "{:#04x} should be in REL_OPCODES",
                    opcode
                ),
            }
        }
    }
}
