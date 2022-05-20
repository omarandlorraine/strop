use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::DyadicOperation;
use crate::machine::DyadicOperation::{Add, AddWithCarry};
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Width;
use crate::machine::R;

use rand::random;
use strop::randomly;

const A: Datum = Datum::Register(R::A);
const B: Datum = Datum::Register(R::B);

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

    fn monadic(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        match d {
            Datum::Absolute(address) => {
                write!(f, "\t{} {}", s, address)
            }
            Datum::Register(R::A) => {
                write!(f, "\t{}a", s)
            }
            Datum::Register(R::B) => {
                write!(f, "\t{}b", s)
            }
            _ => {
                write!(f, "\t{} {:?}", s, d)
            }
        }
    }

    fn dyadic(
        f: &mut std::fmt::Formatter,
        s: &'static str,
        r: Datum,
        d: Datum,
    ) -> std::fmt::Result {
        let r = match r {
            Datum::Register(r) => r,
            _ => panic!(),
        };

        match d {
            Datum::Absolute(address) => {
                write!(f, "\t{}{} {}", s, regname(r), address)
            }
            Datum::Imm8(value) => {
                write!(f, "\t{}{} #{}", s, regname(r), value)
            }
            _ => {
                panic!()
            }
        }
    }

    match op {
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\tt{}{}", regname(from), regname(to))
        }
        Operation::Move(Datum::Register(from), to) => dyadic(f, "sta", Datum::Register(from), to),
        Operation::DecimalAdjustAccumulator => {
            write!(f, "\tdaa")
        }
        Operation::Dyadic(Width::Width8, Add, A, B, A) => {
            write!(f, "\taba")
        }
        Operation::Dyadic(Width::Width8, Add, _, d, r) => dyadic(f, "add", r, d),
        Operation::Dyadic(Width::Width8, AddWithCarry, _, d, r) => dyadic(f, "adc", r, d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => monadic(f, "asl", d),
        Operation::Shift(ShiftType::RightArithmetic, d) => monadic(f, "lsr", d),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, d) => monadic(f, "rol", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => monadic(f, "ror", d),
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

fn add_6800() -> Operation {
    let dst = random_accumulator_6800();

    fn src(op: DyadicOperation, dst: Datum) -> Datum {
        if op == Add && dst == Datum::Register(R::A) && random() {
            Datum::Register(R::B)
        } else {
            random_source_6800()
        }
    }

    let ops = vec![Add, AddWithCarry];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();
    Operation::Dyadic(Width::Width8, op, dst, src(op, dst), dst)
}

fn transfers_6800() -> Operation {
    if random() {
        Operation::Move(Datum::Register(R::A), Datum::Register(R::B))
    } else {
        Operation::Move(Datum::Register(R::B), Datum::Register(R::A))
    }
}

fn rotates_6800() -> Operation {
    randomly!(
        { Operation::Shift(ShiftType::LeftArithmetic, rmw_datum_6800())}
        { Operation::Shift(ShiftType::RightArithmetic, rmw_datum_6800())}
        { Operation::Shift(ShiftType::LeftRotateThroughCarry, rmw_datum_6800())}
        { Operation::Shift(ShiftType::RightRotateThroughCarry, rmw_datum_6800())}
    )
}

fn length(_insn: &Instruction) -> usize {
    1 // TODO!
}

fn reg_by_name(name: &str) -> Result<Datum, &'static str> {
    match name {
        "a" => Ok(A),
        "b" => Ok(B),
        _ => todo!(),
    }
}

pub fn instr_6800() -> Instruction {
    randomly!(
        { Instruction::new(add_6800, dasm, length)}
        { Instruction::new(transfers_6800, dasm, length)}
        { Instruction::new(|| Operation::DecimalAdjustAccumulator, dasm, length)}
        { Instruction::new(rotates_6800, dasm, length)}
    )
    // TODO: Add clc, sec, daa, and many other instructions
}

pub const M6800: Machine = Machine {
    name: "6800",
    description: "Motorola 6800",
    random_insn: instr_6800,
    reg_by_name,
};
