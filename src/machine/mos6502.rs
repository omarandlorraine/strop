use crate::machine::Instruction;
use std::collections::HashMap;

// some clippy warnings disabled for this module because 6502 support is not there yet.

#[derive(Default)]
#[allow(dead_code, unused_variables)]
pub struct Mos6502 {
    a: Option<u8>,
    x: Option<u8>,
    y: Option<u8>,
    s: Option<u8>,
    heap: HashMap<u16, Option<u8>>,
}

#[derive(Clone, Copy)]
pub struct Instruction6502 {
    randomizer: fn(&mut Instruction6502),
    disassemble: fn(&Instruction6502, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut Mos6502),
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl Instruction for Instruction6502 {
    type State = Mos6502;
    fn randomize(&mut self) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, _s: &mut Mos6502) {
        todo!()
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    extern crate mos6502;
    use mos6502::address::Address;
    use mos6502::cpu;
    use mos6502::registers::Status;

    #[allow(dead_code)]
    fn run_mos6502(
        opcode: u8,
        val1: u8,
        val2: u8,
        carry: bool,
        decimal: bool,
    ) -> (i8, bool, bool, bool, bool) {
        let mut cpu = cpu::CPU::new();

        let program = [
            // set or clear the carry flag
            if carry { 0x38 } else { 0x18 },
            // set or clear the decimal flag
            if decimal { 0xf8 } else { 0xd8 },
            // load val1 into the accumulator
            0xa9,
            val1,
            // run the opcode on val2
            opcode,
            val2,
            // stop the emulated CPU
            0xff,
        ];

        cpu.memory.set_bytes(Address(0x10), &program);
        cpu.registers.program_counter = Address(0x10);
        cpu.run();

        (
            cpu.registers.accumulator,
            cpu.registers.status.contains(Status::PS_ZERO),
            cpu.registers.status.contains(Status::PS_CARRY),
            cpu.registers.status.contains(Status::PS_NEGATIVE),
            cpu.registers.status.contains(Status::PS_OVERFLOW),
        )
    }
}
