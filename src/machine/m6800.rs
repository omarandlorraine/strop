use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::R;

use crate::machine::rand::Rng;
use rand::random;

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::B => "b",
            R::Xl => "x",
            R::Yl => "y",
            _ => unimplemented!(),
        }
    }
    match op {
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\tt{}{}", regname(from), regname(to))
        }
        Operation::Add(Datum::Register(R::B), Datum::Register(R::A), false) => {
            write!(f, "\taba")
        }
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn random_accumulator_6800() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        Datum::Register(R::B)
    }
}

fn random_source_6800() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn rmw_datum_6800() -> Datum {
    // Data that can be the operand for 6800 operations like ASL and COM
    if random() {
        random_accumulator_6800()
    } else {
        random_absolute()
    }
}

fn add_6800(_mach: Machine) -> Operation {
    let dst = random_accumulator_6800();
    if dst == Datum::Register(R::A) && random() {
        Operation::Add(Datum::Register(R::B), dst, false) // ABA
    } else {
        Operation::Add(random_source_6800(), dst, random()) // ADCA, ADCB, ADDA, ADDB
    }
}

fn transfers_6800(_mach: Machine) -> Operation {
    if random() {
        Operation::Move(Datum::Register(R::A), Datum::Register(R::B))
    } else {
        Operation::Move(Datum::Register(R::B), Datum::Register(R::A))
    }
}

fn rotates_6800(_mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 4) {
        0 => Operation::Shift(ShiftType::LeftArithmetic, rmw_datum_6800()),
        1 => Operation::Shift(ShiftType::RightArithmetic, rmw_datum_6800()),
        2 => Operation::Shift(ShiftType::LeftRotateThroughCarry, rmw_datum_6800()),
        _ => Operation::Shift(ShiftType::RightRotateThroughCarry, rmw_datum_6800()),
    }
}

pub fn instr_6800(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 4) {
        0 => Instruction::new(mach, add_6800, dasm),
        1 => Instruction::new(mach, transfers_6800, dasm),
        2 => Instruction::new(mach, |_| Operation::DecimalAdjustAccumulator, dasm),
        _ => Instruction::new(mach, rotates_6800, dasm),
    }
    // TODO: Add clc, sec, daa, and many other instructions
}
