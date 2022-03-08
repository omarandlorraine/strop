use crate::machine::Instruction;
use crate::machine::R;
use crate::machine::Machine;
use crate::machine::Mos6502Variant;
use crate::machine::Operation;
use crate::machine::Datum;
use crate::machine::ShiftType;
use crate::machine::random_immediate;
use crate::machine::random_absolute;

use crate::machine::rand::Rng;
use rand::random;

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::Xl => "x",
            R::Yl => "y",
            _ => unimplemented!()
        }
    }

    match op {
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\tt{}{}", regname(from), regname(to))
        }
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn random_source_6502() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn incdec_6502(mach: Machine) -> Operation {
    // the CMOS varieties have inc and dec for accumulator
    // but earlier 6502s can increment and decrement X and Y only.
    let reg = 
        match rand::thread_rng().gen_range(0, if mach == Machine::Mos6502(Mos6502Variant::Cmos) { 3 } else { 2 }) {
            0 => {Datum::Register(R::Xl)}
            1 => {Datum::Register(R::Yl)}
            _ => {Datum::Register(R::A)}
        };
    if random() {
        Operation::Increment(reg)
    } else {
        Operation::Decrement(reg)
    }
}

fn add_6502(_mach: Machine) -> Operation {
    Operation::Add(random_source_6502(), Datum::Register(R::A), true)
}

fn transfers_6502(_mach: Machine) -> Operation {
    let reg = if random() {
        Datum::Register(R::Xl)
    } else {
        Datum::Register(R::Yl)
    };
    if random() {
        Operation::Move(Datum::Register(R::A), reg)
    } else {
        Operation::Move(reg, Datum::Register(R::A))
    }
}

fn loadstore_6502(mach: Machine) -> Operation {
    // TODO: STZ operation for CMOS varieties
    let addr = random_absolute();
    let reg = match rand::thread_rng().gen_range(0, if mach == Machine::Mos6502(Mos6502Variant::Cmos) { 4 } else { 3 }) {
        0 => Datum::Register(R::A),
        1 => Datum::Register(R::Xl),
        2 => Datum::Register(R::Yl),
        _ => Datum::Zero,
    };
    if random() && reg != Datum::Zero {
        Operation::Move(addr, reg)
    } else {
        Operation::Move(reg, addr)
    }
}

fn secl_6502(_mach: Machine) -> Operation {
    Operation::Carry(random())
}
fn shifts_6502(_mach: Machine) -> Operation {
    let sht = match rand::thread_rng().gen_range(0, 4) {
        0 => ShiftType::LeftArithmetic,
        1 => ShiftType::RightArithmetic,
        2 => ShiftType::LeftRotateThroughCarry,
        _ => ShiftType::RightRotateThroughCarry,
    };
    let dat = if random() {
        Datum::Register(R::A)
    }else {
        random_absolute()
    };
    Operation::Shift(sht, dat)
}

pub fn instr_6502(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 5) {
        0 => Instruction::new(mach, incdec_6502, dasm),
        1 => Instruction::new(mach, add_6502, dasm),
        2 => Instruction::new(mach, transfers_6502, dasm),
        3 => Instruction::new(mach, shifts_6502, dasm),
        4 => Instruction::new(mach, loadstore_6502, dasm),
        _ => Instruction::new(mach, secl_6502, dasm),
    }
    // TODO: Add clc, sec, and many other instructions
}
