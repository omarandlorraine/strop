use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Mos6502Variant;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::R;

use crate::machine::rand::Rng;
use rand::random;

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::Xl => "x",
            R::Yl => "y",
            _ => unimplemented!(),
        }
    }

    fn syn(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        match d {
            Datum::Absolute(address) => {
                write!(f, "\t{} {}", s, address)
            }
            Datum::Register(R::A) => {
                write!(f, "\t{} a", s)
            }
            _ => {
                write!(f, "\t{} {:?}", s, d)
            }
        }
    }

    match op {
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            // tax, txa, tay, tya, txs, tsx, txy, tyx (the latter two exist on 65816)
            write!(f, "\tt{}{}", regname(from), regname(to))
        }
        Operation::Move(Datum::Register(R::A), thing) => syn(f, "sta", thing),
        Operation::Move(Datum::Register(R::Xl), thing) => syn(f, "stx", thing),
        Operation::Move(Datum::Register(R::Yl), thing) => syn(f, "sty", thing),
        Operation::Move(Datum::Zero, thing) => syn(f, "stz", thing),
        Operation::Move(thing, Datum::Register(R::A)) => syn(f, "lda", thing),
        Operation::Move(thing, Datum::Register(R::Xl)) => syn(f, "ldx", thing),
        Operation::Move(thing, Datum::Register(R::Yl)) => syn(f, "ldy", thing),
        Operation::Shift(ShiftType::RightArithmetic, thing) => syn(f, "lsr", thing),
        Operation::Shift(ShiftType::LeftArithmetic, thing) => syn(f, "asl", thing),
        Operation::Shift(ShiftType::RightRotateThroughCarry, thing) => syn(f, "ror", thing),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, thing) => syn(f, "rol", thing),
        Operation::Add(thing, Datum::Register(R::A), true) => syn(f, "adc", thing),
        Operation::Increment(Datum::Register(reg)) => write!(f, "\tin{}", regname(reg)),
        Operation::Decrement(Datum::Register(reg)) => write!(f, "\tde{}", regname(reg)),
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
    let reg = match rand::thread_rng().gen_range(
        0,
        if mach == Machine::Mos6502(Mos6502Variant::Cmos) {
            3
        } else {
            2
        },
    ) {
        0 => Datum::Register(R::Xl),
        1 => Datum::Register(R::Yl),
        _ => Datum::Register(R::A),
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
    let reg = match rand::thread_rng().gen_range(
        0,
        if mach == Machine::Mos6502(Mos6502Variant::Cmos) {
            4
        } else {
            3
        },
    ) {
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
    } else {
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
