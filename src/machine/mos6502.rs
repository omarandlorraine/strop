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
    decimal: Option<bool>,
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

fn rmwop_randomizer(insn: &mut Instruction6502) {
    fn rnd() -> Operand6502 {
        randomly!(
            {Operand6502::A}
            {Operand6502::Absolute(random())}
        )
    }

    insn.operand = match insn.operand {
        Operand6502::A => rnd(),
        Operand6502::Immediate(_) => rnd(),
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
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let addition = a
            .zip(m)
            .zip(s.carry)
            .map(|((a, m), c)| a.wrapping_add(m).wrapping_add(if c { 1 } else { 0 }));

        let decimal_adjust = s.decimal.zip(addition).map(|(d, q)| {
            let r = u8::from_ne_bytes(q.to_ne_bytes());
            if d {
                let s1 = if r & 0x0f > 9 { r.wrapping_add(6) } else { r };
                if s1 & 0xf0 > 0x90 {
                    s.carry = Some(true);
                    s1.wrapping_add(0x60)
                } else {
                    s.carry = Some(false);
                    s1
                }
            } else {
                r
            }
        });
        let r = decimal_adjust.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
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

const AND: Instruction6502 = Instruction6502 {
    mnem: "and",
    randomizer: aluop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const ASL: Instruction6502 = Instruction6502 {
    mnem: "asl",
    randomizer: rmwop_randomizer,
    disassemble,
    operand: Operand6502::Immediate(0),
    handler: |insn, s| {
        let val = insn.operand.get(s);
        let m = val.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let a = s.a.map(|v| i8::from_ne_bytes(v.to_ne_bytes()));
        let r = a.zip(m).map(|(a, m)| a & m);
        s.zero = r.map(|r| r == 0);
        s.sign = r.map(|r| r.leading_zeros() == 0);
        s.a = r.map(|v| u8::from_ne_bytes(v.to_ne_bytes()));
    },
};

const INSTRUCTIONS: [Instruction6502; 3] = [ADC, AND, ASL];

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
        let mut insn = *INSTRUCTIONS.choose(&mut rand::thread_rng()).unwrap();
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

    fn run_strop(
        instr: Instruction6502,
        val1: u8,
        val2: Option<u8>,
        carry: bool,
        decimal: bool,
    ) -> (i8, bool, bool, bool, bool) {
        let mut state: Mos6502 = Default::default();
        state.carry = Some(carry);
        state.decimal = Some(decimal);
        state.a = Some(val1);
        let mut insn = instr;
        insn.operand = if let Some(v) = val2 {
            Operand6502::Immediate(v)
        } else {
            Operand6502::A
        };

        insn.operate(&mut state);
        (
            i8::from_ne_bytes(state.a.unwrap().to_ne_bytes()),
            state.zero.unwrap_or(false),
            state.carry.unwrap_or(false),
            state.sign.unwrap_or(false),
            state.overflow.unwrap_or(false),
        )
    }

    fn fuzz_test_immediate(insn: &Instruction6502, opcode: u8) {
        for _ in 0..5000 {
            let a: u8 = random();
            let b: u8 = random();
            let c: bool = random();
            let d: bool = random();
            let t = run_mos6502(opcode, a, b, c, d);
            let s = run_strop(*insn, a, Some(b), c, d);

            let regr = format!(
                "run_strop({}, {:#04x}, Some({:#04x}), {}, {})",
                insn.mnem.to_ascii_uppercase(),
                a,
                b,
                c,
                d
            );

            assert!(t.0 == s.0, "assert!({}.0 == {:#04x})", regr, t.0);
            assert!(t.1 == s.1, "assert!({}.1 == {})", regr, t.1);
            assert!(t.2 == s.2, "assert!({}.2 == {})", regr, t.2);
            assert!(t.3 == s.3, "assert!({}.3 == {})", regr, t.3);
        }
    }

    fn fuzz_test_implied(insn: &Instruction6502, opcode: u8) {
        for _ in 0..5000 {
            let a: u8 = random();
            let b: u8 = random();
            let c: bool = random();
            let d: bool = random();
            let t = run_mos6502(opcode, a, 0xea, c, d);
            let s = run_strop(*insn, a, None, c, d);

            let regr = format!(
                "run_strop({}, {:#04x}, Some({:#04x}), {}, {})",
                insn.mnem.to_ascii_uppercase(),
                a,
                b,
                c,
                d
            );

            assert!(t.0 == s.0, "assert!({}.0 == {:#04x})", regr, t.0);
            assert!(t.1 == s.1, "assert!({}.1 == {})", regr, t.1);
            assert!(t.2 == s.2, "assert!({}.2 == {})", regr, t.2);
            assert!(t.3 == s.3, "assert!({}.3 == {})", regr, t.3);
        }
    }

    #[test]
    fn fuzz_and() {
        fuzz_test_immediate(&AND, 0x29);
    }

    #[test]
    fn fuzz_adc() {
        fuzz_test_immediate(&ADC, 0x69);
    }

    #[test]
    fn fuzz_asl() {
        fuzz_test_implied(&ASL, 0x69);
    }

    #[test]
    fn decimal_regression_tests() {
        assert!(run_strop(ADC, 0x05, Some(0x05), false, true).0 == 0x10);
        assert!(run_strop(ADC, 0x03, Some(0xfa), true, true).0 == 0x04);
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
        for opcode in vec![
            "adc", "and", "asl", "bit", "clc", "cld", "clv", "cmp", "cpx", "cpy", "dec", "dex",
            "dey", "eor", "inc", "inx", "iny", "lda", "ldx", "ldy", "lsr", "ora", "pha", "pla",
            "rol", "ror", "sbc", "sec", "sed", "sta", "stx", "sty", "tax", "tay", "tsx", "txa",
            "txs", "tya",
        ] {
            find_it(opcode);
            // todo: execute these instructions and check that they don't set the CMOS flag
        }
    }

    #[test]
    fn instruction_set_illegal() {
        // I've taken the list from https://www.masswerk.at/nowgobang/2021/6502-illegal-opcodes
        for opcode in vec![
            "alr", "anc", "arr", "dcp", "isc", "las", "lax", "rla", "rra", "sax", "sbx", "slo",
            "sre",
        ] {
            find_it(opcode);
            // todo: execute these instructions and check that they set the illegal flag
        }
    }

    #[test]
    fn instruction_set_cmos() {
        for opcode in &["phx", "phy", "plx", "ply", "stz", "trb", "tsb"] {
            find_it(opcode);
            // todo: execute these instructions and check that they set the CMOS flag
        }
    }
}
