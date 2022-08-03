use crate::machine::Instruction;
use crate::machine::Strop;
use rand::random;
use randomly::randomly;
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
    carry: Option<bool>,
    zero: Option<bool>,
    sign: Option<bool>,
    overflow: Option<bool>,
}

impl Mos6502 {
    fn read_mem(&self, addr: u16) -> Option<u8> {
        *self.heap.get(&addr).unwrap_or(&None)
    }
}

#[derive(Clone, Copy)]
pub enum Operand6502 {
    A,
    Immediate(u8),
    Absolute(u16),
}

impl Operand6502 {
    fn get(self, s: &Mos6502) -> Option<u8> {
        match self {
            Operand6502::A => s.a,
            Operand6502::Immediate(v) => Some(v),
            Operand6502::Absolute(addr) => s.read_mem(addr),
        }
    }
}

fn aluop_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        randomly!(
            {Operand6502::Immediate(random())}
            {Operand6502::Absolute(random())}
        )
    }

    insn.operand = match insn.operand {
        Operand6502::A => rnd(),
        Operand6502::Immediate(v) => {
            randomly!(
                {Operand6502::Immediate(v.wrapping_add(1))}
                {Operand6502::Immediate(v.wrapping_sub(1))}
                {let bitsel = 1_u8.rotate_left(rand::thread_rng().gen_range(0..8)); Operand6502::Immediate(v ^ bitsel)}
            )
        }
        Operand6502::Absolute(addr) => {
            randomly!(
                {Operand6502::Absolute(addr.wrapping_add(1))}
                {Operand6502::Absolute(addr.wrapping_sub(1))}
            )
        }
    }
}

fn disassemble(insn: &Instruction6502, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match insn.operand {
        Operand6502::A => {
            write!(f, "\t{} a", insn.mnem)
        }
        Operand6502::Immediate(val) => {
            write!(f, "\t{} #${:#04x}", insn.mnem, val)
        }
        Operand6502::Absolute(addr) => {
            write!(f, "\t{} ${:#06x}", insn.mnem, addr)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Instruction6502 {
    mnem: &'static str,
    randomizer: fn(&mut Instruction6502),
    disassemble: fn(&Instruction6502, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut Mos6502),
    operand: Operand6502,
}

const ADC: Instruction6502 = Instruction6502 {
    mnem: "adc",
    randomizer: aluop_randomizer,
    disassemble: disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a
            .zip(m)
            .zip(s.carry)
            .map(|((a, m), c)| a.wrapping_add(m).wrapping_add(if c { 1 } else { 0 }));
        let carrytests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| (a & m) | (m & !r) | (!r & a));
        let overflowtests = a
            .zip(m)
            .zip(r)
            .map(|((a, m), r)| ((a & m) | (m & r) | (r & a)) & -64);
        s.carry = carrytests.map(|t| t.leading_zeros() == 0);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.overflow = overflowtests.map(|t| t != 0 && t != -64);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const Instructions: [Instruction6502; 1] = [ADC];

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.disassemble)(self, f)
    }
}

impl Instruction for Instruction6502 {
    type State = Mos6502;
    fn randomize(&mut self) {
        (self.randomizer)(self);
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, s: &mut Mos6502) {
        (self.handler)(self, s);
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        use rand::seq::SliceRandom;
        let mut insn = Instructions
            .choose(&mut rand::thread_rng())
            .unwrap()
            .clone();
        insn.randomize();
        insn
    }
}

impl Strop for Instruction6502 {
    fn random() -> Instruction6502 {
        Instruction6502::new()
    }

    fn mutate(&mut self) {
        (self.randomizer)(self);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use mos6502;
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

    fn find_it(opcode: &'static str) {
        for _ in 0..5000 {
            let insn = Instruction6502::random();
            let dasm = format!("{}", insn);
            if dasm.contains(opcode) {
                return;
            }
        }
        panic!("Could not find opcode {}", opcode);
    }

    #[test]
    fn instruction_set() {
        for opcode in vec!["adc", "asl"] {
            find_it(opcode);
        }
    }
}
