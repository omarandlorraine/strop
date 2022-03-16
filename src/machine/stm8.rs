use crate::Machine;
use crate::machine::Instruction;
use crate::Datum;
use crate::machine::R;
use crate::machine::random_immediate;
use crate::machine::random_absolute;
use crate::machine::Operation;
use crate::machine::ShiftType;

use crate::machine::rand::Rng;
use rand::random;

fn random_stm8_operand() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn random_register() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        if random() {
            Datum::RegisterPair(R::Xh, R::Xl)
        } else {
            Datum::RegisterPair(R::Xh, R::Xl)
        }
    }
}

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn syn(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        let t = match d {
            Datum::Imm8(val) => format!("#${:2}", val),
            Datum::Absolute(addr) if addr < 256 => format!("${:2}", addr),
            Datum::Absolute(addr) => format!("${:4}", addr),
            Datum::Register(R::A) => "a".into(),
            _ => format!("{:?}", d),
        };
        write!(f, "\t{} {}", s, t)
    }

    match op {
        Operation::Add(d, Datum::Register(R::A), true) => syn(f, "adc", d),
        Operation::Add(d, Datum::Register(R::A), false) => syn(f, "add", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => syn(f, "rrc", d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => syn(f, "sla", d),
        _ => write!(f, "{:?}", op)
    }
}

fn add_adc(_mach: Machine) -> Operation {
    Operation::Add(random_stm8_operand(), Datum::Register(R::A), random())
}

fn shifts(_mach: Machine) -> Operation {
    // TODO: instructions SRA or SRAW.
    // TODO: instructions RLWA or RRWA.
    let sht = match rand::thread_rng().gen_range(0, 2) {
        0 => ShiftType::LeftArithmetic,
        1 => ShiftType::RightArithmetic,
        2 => ShiftType::RightRotateThroughCarry,
        _ => ShiftType::LeftRotateThroughCarry
    };

    let operand = if random() {
        random_absolute()
    } else {
        random_register()
    };

    Operation::Shift(sht, operand)
}

pub fn instr_stm8(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 2) {
        0 => Instruction::new(mach, add_adc, dasm),
        _ => Instruction::new(mach, shifts, dasm),
    }
    // TODO: Add clc, sec, and many other instructions
}
