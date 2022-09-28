//! The `Instruction6502` type, for representing a MOS 6502 instruction.

#![warn(missing_debug_implementations, missing_docs)]
#![allow(dead_code)]

use crate::instruction::Instruction;
use dez80::instruction::DecodingState;
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

impl Instruction for InstructionZ80 {
    fn length(&self) -> usize {
        let instr = DeZ80Instruction::decode_one(&mut self.encoding.as_slice()).unwrap();
        instr.to_bytes().len()
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        loop {
            let encoding: [u8; 5] = [random(), random(), random(), random(), random()];
            if let Ok(instr) = DeZ80Instruction::decode_one(&mut encoding.as_slice()) {
                return Self { encoding: encoding };
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
}

#[cfg(test)]
mod test {
    use crate::instruction::Instruction;
    use crate::mos6502::Instruction6502;

    #[test]
    fn new_instructions() {
        for _i in 0..50000 {
            let mut insn = Instruction6502::new();
        }
    }
}
