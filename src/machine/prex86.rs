use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{Add, AddWithCarry};
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Width;
use crate::machine::R;
use rand::random;
use strop::randomly;

fn random_r_prex86() -> Datum {
    randomly!(
        { Datum::Register(R::A)}
        { Datum::Register(R::B)}
        { Datum::Register(R::C)}
        { Datum::Register(R::D)}
        { Datum::Register(R::E)}
        { Datum::Register(R::A)} // TODO: this should be (HL) in the zilog syntax; the byte pointed to by HL
        { Datum::Register(R::H)}
        { Datum::Register(R::L)}
    )
}

fn random_rp_prex86() -> Datum {
    randomly!(
        { Datum::RegisterPair(R::B, R::C)}
        { Datum::RegisterPair(R::D, R::E)}
        { Datum::RegisterPair(R::H, R::L)}
    )
}

fn inc_dec_prex86() -> Operation {
    let (w, r) = if random() {
        (Width::Width8, random_r_prex86())
    } else {
        (Width::Width16, random_rp_prex86())
    };

    randomly!(
        { Operation::Monadic(w, MonadicOperation::Increment, r, r) }
        { Operation::Monadic(w, MonadicOperation::Decrement, r, r) }
    )
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
        Operation::Dyadic(
            Width::Width8,
            Add,
            thing,
            Datum::Register(R::A),
            Datum::Register(R::A),
        ) => {
            write!(f, "\tadd a, {}", name(thing))
        }
        Operation::Dyadic(
            Width::Width8,
            AddWithCarry,
            thing,
            Datum::Register(R::A),
            Datum::Register(R::A),
        ) => {
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
        Operation::Monadic(_, MonadicOperation::Increment, r, _) => monadic(f, "inc", r),
        Operation::Monadic(_, MonadicOperation::Decrement, r, _) => monadic(f, "dec", r),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn add8_prex86() -> Operation {
    // From what I can see, the KR580VM1 and similar CPUs, can do:
    //  - 8 bit adds with or without carry, destination is the Accumulator
    //  - 16 bit add without carry, destination is the HL register pair
    let ops = vec![Add, AddWithCarry];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();

    let args = vec![random_immediate(), random_r_prex86()];
    let arg = *args.choose(&mut rand::thread_rng()).unwrap();

    Operation::Dyadic(
        Width::Width8,
        op,
        arg,
        Datum::Register(R::A),
        Datum::Register(R::A),
    )
}

fn rot_a_prex86() -> Operation {
    randomly!(
        { Operation::Shift(ShiftType::LeftArithmetic, Datum::Register(R::A))}
        { Operation::Shift(ShiftType::RightArithmetic, Datum::Register(R::A))}
    )
}

fn ld_prex86() -> Operation {
    Operation::Move(random_r_prex86(), random_r_prex86())
}

fn registers_8080(name: &str) -> Result<Datum, &'static str> {
    match name {
        "a" => Ok(Datum::Register(R::A)),
        "b" => Ok(Datum::Register(R::B)),
        "c" => Ok(Datum::Register(R::C)),
        "d" => Ok(Datum::Register(R::D)),
        "e" => Ok(Datum::Register(R::E)),
        "h" => Ok(Datum::Register(R::H)),
        "l" => Ok(Datum::Register(R::L)),
        "bc" => Ok(Datum::RegisterPair(R::B, R::C)),
        "de" => Ok(Datum::RegisterPair(R::D, R::E)),
        "hl" => Ok(Datum::RegisterPair(R::H, R::L)),
        _ => {
            panic!("No such register as {}", name);
        }
    }
}

fn registers_kr580vm1(r: &str) -> Result<Datum, &'static str> {
    if r == "h1" {
        Ok(Datum::Register(R::H1))
    } else if r == "l1" {
        Ok(Datum::Register(R::L1))
    } else if r == "h1l1" {
        Ok(Datum::RegisterPair(R::H1, R::L1))
    } else {
        registers_8080(r)
    }
}

fn insn_len(_insn: &Instruction) -> usize {
    1 // TODO!
}

pub fn random_insn_kr580vm1() -> Instruction {
    randomly!(
        { Instruction::new(inc_dec_prex86, dasm, insn_len)}
        { Instruction::new(add8_prex86, dasm, insn_len)}
        { Instruction::new(rot_a_prex86, dasm, insn_len)}
        { Instruction::new(ld_prex86, dasm, insn_len)}
        { Instruction::new(|| Operation::DecimalAdjustAccumulator, dasm, insn_len)}
    )
}

pub const KR580VM1: Machine = Machine {
    name: "kr580vm1",
    random_insn: random_insn_kr580vm1,
    reg_by_name: registers_kr580vm1,
};
