use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{Add, And};
use crate::machine::Instruction;
use crate::machine::MonadicOperation;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Width;
use crate::machine::R;

use rand::random;
use strop::randomly;

const W: Datum = Datum::Register(R::A);

fn dasm(op: Operation, fr: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn dest(d: Datum) -> &'static str {
        match d {
            Datum::Absolute(_) => "1",
            Datum::Register(R::A) => "0",
            _ => panic!(),
        }
    }
    match op {
        Operation::Monadic(_, MonadicOperation::Decrement, Datum::Absolute(f), dst) => {
            write!(fr, "\tdecf {}, {}", f, dest(dst))
        }
        Operation::Monadic(_, MonadicOperation::Increment, Datum::Absolute(f), dst) => {
            write!(fr, "\tincf {}, {}", f, dest(dst))
        }
        Operation::Dyadic(
            Width::Width8,
            And,
            Datum::Absolute(f),
            Datum::Register(R::A),
            Datum::Absolute(_),
        ) => {
            write!(fr, "\tandwf {}, 1", f)
        }
        Operation::Dyadic(
            Width::Width8,
            And,
            Datum::Absolute(f),
            Datum::Register(R::A),
            Datum::Register(R::A),
        ) => {
            write!(fr, "\tandwf {}, 0", f)
        }
        Operation::Dyadic(
            Width::Width8,
            And,
            Datum::Imm8(k),
            Datum::Register(R::A),
            Datum::Register(R::A),
        ) => {
            write!(fr, "\tandlw {}, 0", k)
        }
        Operation::Dyadic(Width::Width8, Add, W, Datum::Imm8(k), W) => {
            write!(fr, "\taddlw {}, 0", k)
        }
        Operation::Dyadic(Width::Width8, Add, W, Datum::Absolute(f), W) => {
            write!(fr, "\taddwf {}, 0", f)
        }
        Operation::Dyadic(Width::Width8, Add, W, _, Datum::Absolute(f)) => {
            write!(fr, "\taddwf {}, 1", f)
        }
        Operation::Move(Datum::Absolute(f), Datum::Register(R::A)) => {
            write!(fr, "\tmovf {}, 0", f)
        }
        Operation::Move(Datum::Register(R::A), Datum::Absolute(f)) => {
            write!(fr, "\tmovwf {}", f)
        }
        Operation::Move(Datum::Zero, Datum::Absolute(f)) => {
            write!(fr, "\tclrf {}", f)
        }
        Operation::Move(Datum::Zero, Datum::Register(R::A)) => {
            write!(fr, "\tclrw")
        }
        Operation::Move(Datum::Imm8(k), Datum::Register(R::A)) => {
            write!(fr, "\tmovlw {}", k)
        }
        Operation::Shift(ShiftType::LeftRotateThroughCarry, Datum::Absolute(f)) => {
            write!(fr, "\trlf {}, 1", f)
        }
        Operation::Shift(ShiftType::RightRotateThroughCarry, Datum::Absolute(f)) => {
            write!(fr, "\trrf {}, 1", f)
        }
        _ => write!(fr, "{:?}", op),
    }
}

fn random_accumulator_or_absolute() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        random_absolute()
    }
}

fn inc_dec_pic() -> Operation {
    let src = random_absolute();

    let dst = if random() { src } else { Datum::Register(R::A) };

    randomly!(
        { Operation::Monadic(Width::Width8, MonadicOperation::Increment, src, dst) }
        { Operation::Monadic(Width::Width8, MonadicOperation::Decrement, src, dst) }
    )
}

fn add_pic() -> Operation {
    let dst = random_accumulator_or_absolute();
    if dst == Datum::Register(R::A) {
        // This is an immediate add (addlw). Not available on PIC12.
        Operation::Dyadic(Width::Width8, Add, W, random_immediate(), W)
    } else if random() {
        Operation::Dyadic(Width::Width8, Add, W, random_absolute(), W) // addwf f, 0
    } else {
        let f = random_absolute();
        Operation::Dyadic(Width::Width8, Add, W, f, f) // addwf f, 1
    }
}

fn shifts_pic() -> Operation {
    // TODO: These instructions can optionally write to W instead of the F.
    let shtype = if random() {
        ShiftType::RightRotateThroughCarry
    } else {
        ShiftType::LeftRotateThroughCarry
    };
    Operation::Shift(shtype, random_absolute()) // rlf f,d and rrf f,d
}

fn and_pic() -> Operation {
    let w = Datum::Register(R::A);
    let imm = random_immediate();
    let abs = random_absolute();

    randomly!(
        { Operation::Dyadic(Width::Width8, And, imm, w, w)  /* andlw */ }
        { Operation::Dyadic(Width::Width8, And, abs, w, w)  /* andwf something, 0 */ }
        { Operation::Dyadic(Width::Width8, And, abs, w, abs)  /* andwf something, 1 */ }
    )
}

fn store_pic() -> Operation {
    // TODO: There also is movf f,d, which just updates the Z flag
    randomly!(
        { Operation::Move(Datum::Zero, random_accumulator_or_absolute())} // clrw and clrf f
        { Operation::Move(random_absolute(), Datum::Register(R::A))}      // movf f
        { Operation::Move(random_immediate(), Datum::Register(R::A))}     // movlw k
        { Operation::Move(Datum::Register(R::A), random_absolute())}      // movwf f
    )
}

fn insn_len(_insn: &Instruction) -> usize {
    1
}

pub fn instr_pic() -> Instruction {
    randomly!(
        { Instruction::new(shifts_pic, dasm, insn_len)}
        { Instruction::new(and_pic, dasm, insn_len)}
        { Instruction::new(add_pic, dasm, insn_len)}
        { Instruction::new(store_pic, dasm, insn_len)}
        { Instruction::new(inc_dec_pic, dasm, insn_len)}
    )
}
