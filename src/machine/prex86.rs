use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::R;

use crate::machine::rand::Rng;
use rand::random;

fn random_r_prex86(_mach: Machine) -> Datum {
    match rand::thread_rng().gen_range(0, 8) {
        0 => Datum::Register(R::A),
        1 => Datum::Register(R::B),
        2 => Datum::Register(R::C),
        3 => Datum::Register(R::D),
        4 => Datum::Register(R::E),
        5 => Datum::Register(R::A), // TODO: this should be (HL) in the zilog syntax; the byte pointed to by HL.
        6 => Datum::Register(R::H),
        _ => Datum::Register(R::L),
    }
}

fn random_rp_prex86(_mach: Machine) -> Datum {
    match rand::thread_rng().gen_range(0, 3) {
        0 => Datum::RegisterPair(R::B, R::C),
        1 => Datum::RegisterPair(R::D, R::E),
        _ => Datum::RegisterPair(R::H, R::L),
    }
}

fn inc_dec_prex86(mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 4) {
        0 => Operation::Increment(random_r_prex86(mach)),
        1 => Operation::Increment(random_rp_prex86(mach)),
        2 => Operation::Decrement(random_r_prex86(mach)),
        _ => Operation::Decrement(random_rp_prex86(mach)),
    }
}

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn name(datum: Datum) -> &'static str {
        match datum {
            Datum::Register(R::A) => "a",
            Datum::Register(R::B) => "b",
            Datum::Register(R::C) => "c",
            Datum::Register(R::D) => "d",
            Datum::Register(R::E) => "e",
            Datum::Register(R::H) => "h",
            Datum::Register(R::L) => "l",
            _ => "<something>",
        }
    }

    fn rpname(a: R, b: R) -> &'static str {
        match (a, b) {
            (R::H, R::L) => "hl",
            (R::B, R::C) => "bc",
            (R::D, R::E) => "de",
            (R::H1, R::L1) => "h1l1",
            _ => unimplemented!(),
        }
    }

    fn monadic(
        f: &mut std::fmt::Formatter<'_>,
        ins: &'static str,
        operand: Datum,
    ) -> std::fmt::Result {
        match operand {
            Datum::Register(_) => write!(f, "\t{} {}", ins, name(operand)),
            Datum::RegisterPair(a, b) => write!(f, "\t{} {}", ins, rpname(a, b)),
            _ => {
                unimplemented!()
            }
        }
    }

    match op {
        Operation::Add(thing, Datum::Register(R::A), false) => {
            write!(f, "\tadd a, {}", name(thing))
        }
        Operation::Add(thing, Datum::Register(R::A), true) => {
            write!(f, "\tadc a, {}", name(thing))
        }
        Operation::Move(from, to) => {
            write!(f, "\tld {}, {}", name(from), name(to))
        }
        Operation::Shift(ShiftType::LeftArithmetic, operand) => {
            write!(f, "\tsla {}", name(operand))
        }
        Operation::Shift(ShiftType::RightArithmetic, operand) => {
            write!(f, "\tsra {}", name(operand))
        }
        Operation::DecimalAdjustAccumulator => {
            write!(f, "\tdaa")
        }
        Operation::Decrement(d) => monadic(f, "dec", d),
        Operation::Increment(d) => monadic(f, "inc", d),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn add8_prex86(mach: Machine) -> Operation {
    // From what I can see, the KR580VM1 and similar CPUs, can do:
    //  - 8 bit adds with or without carry, destination is the Accumulator
    //  - 16 bit add without carry, destination is the HL register pair
    match rand::thread_rng().gen_range(0, 2) {
        0 => Operation::Add(random_immediate(), Datum::Register(R::A), random()), // immediate add
        _ => Operation::Add(random_r_prex86(mach), Datum::Register(R::A), random()), // add register
    }
}

fn rot_a_prex86(_mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 2) {
        0 => Operation::Shift(ShiftType::LeftArithmetic, Datum::Register(R::A)),
        _ => Operation::Shift(ShiftType::RightArithmetic, Datum::Register(R::A)),
    }
}

fn ld_prex86(mach: Machine) -> Operation {
    Operation::Move(random_r_prex86(mach), random_r_prex86(mach))
}

pub fn instr_prex86(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 5) {
        0 => Instruction::new(mach, inc_dec_prex86, dasm),
        1 => Instruction::new(mach, add8_prex86, dasm),
        2 => Instruction::new(mach, rot_a_prex86, dasm),
        3 => Instruction::new(mach, ld_prex86, dasm),
        _ => Instruction::new(mach, |_| Operation::DecimalAdjustAccumulator, dasm),
    }
}
