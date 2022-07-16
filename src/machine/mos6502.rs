use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::reg_by_name;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{
    AddWithCarry, And, ExclusiveOr, Or, Subtract, SubtractWithCarry,
};
use crate::machine::FlowControl;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::MonadicOperation::{
    Decrement, Increment, LeftShiftArithmetic, RightShiftLogical, RotateLeftThruCarry,
    RotateRightThruCarry,
};
use crate::machine::Operation;
use crate::machine::Test;
use crate::machine::Test::{Carry, Minus, Overflow, True, Zero};
use crate::machine::Width;
use crate::machine::R;
use crate::State;

use rand::random;
use strop::randomly;

#[derive(Clone, Copy)]
pub struct Instruction6502 {
    randomizer: fn() -> Operation,
    disassemble: fn(Operation, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    handler: fn(&Instruction6502, &mut State) -> FlowControl,
}

impl std::fmt::Display for Instruction6502 {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl Instruction for Instruction6502 {
    fn randomize(&mut self) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, s: &mut State) -> FlowControl {
        todo!()
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

const A: Datum = Datum::Register(R::A);
const X: Datum = Datum::Register(R::Xl);
const Y: Datum = Datum::Register(R::Yl);

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::Xl => "x",
            R::Yl => "y",
            _ => unimplemented!(),
        }
    }

    fn branch(f: &mut std::fmt::Formatter, s: &'static str, d: FlowControl) -> std::fmt::Result {
        match d {
            FlowControl::Backward(o) => {
                write!(f, "\t{} -{}", s, o)
            }
            FlowControl::Forward(o) => {
                write!(f, "\t{} +{}", s, o)
            }
            _ => {
                write!(f, "\t{} {:?}", s, d)
            }
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
            Datum::Imm8(val) => {
                write!(f, "\t{} #{}", s, val)
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
        Operation::Move(A, thing) => syn(f, "sta", thing),
        Operation::Move(X, thing) => syn(f, "stx", thing),
        Operation::Move(Y, thing) => syn(f, "sty", thing),
        Operation::Move(Datum::Zero, thing) => syn(f, "stz", thing),
        Operation::Move(thing, A) => syn(f, "lda", thing),
        Operation::Move(thing, X) => syn(f, "ldx", thing),
        Operation::Move(thing, Y) => syn(f, "ldy", thing),
        Operation::BitCompare(thing, A) => syn(f, "bit", thing),
        Operation::Dyadic(Width::Width8, Subtract, A, thing, Datum::Zero) => syn(f, "cmp", thing),
        Operation::Dyadic(Width::Width8, Subtract, X, thing, Datum::Zero) => syn(f, "cpx", thing),
        Operation::Dyadic(Width::Width8, Subtract, Y, thing, Datum::Zero) => syn(f, "cpy", thing),
        Operation::Dyadic(Width::Width8, SubtractWithCarry, A, thing, A) => syn(f, "sbc", thing),
        Operation::Dyadic(Width::Width8, AddWithCarry, A, thing, A) => syn(f, "adc", thing),
        Operation::Dyadic(Width::Width8, And, A, thing, A) => syn(f, "and", thing),
        Operation::Dyadic(Width::Width8, ExclusiveOr, A, thing, A) => syn(f, "eor", thing),
        Operation::Dyadic(Width::Width8, Or, A, thing, A) => syn(f, "ora", thing),
        Operation::Monadic(Width::Width8, LeftShiftArithmetic, d, _) => syn(f, "asl", d),
        Operation::Monadic(Width::Width8, MonadicOperation::Increment, Datum::Register(r), _) => {
            write!(f, "\tin{}", regname(r))
        }
        Operation::Monadic(Width::Width8, MonadicOperation::Decrement, Datum::Register(r), _) => {
            write!(f, "\tde{}", regname(r))
        }
        Operation::Monadic(Width::Width8, MonadicOperation::Increment, dat, _) => {
            syn(f, "inc", dat)
        }
        Operation::Monadic(Width::Width8, MonadicOperation::Decrement, dat, _) => {
            syn(f, "dec", dat)
        }
        Operation::Overflow(false) => write!(f, "\tclv"),
        Operation::Carry(false) => write!(f, "\tclc"),
        Operation::Carry(true) => write!(f, "\tsec"),
        Operation::Decimal(false) => write!(f, "\tcld"),
        Operation::Decimal(true) => write!(f, "\tsed"),
        Operation::Jump(Overflow(false), offs) => branch(f, "bvc", offs),
        Operation::Jump(Overflow(true), offs) => branch(f, "bvs", offs),
        Operation::Jump(Zero(false), offs) => branch(f, "bne", offs),
        Operation::Jump(Zero(true), offs) => branch(f, "beq", offs),
        Operation::Jump(Minus(false), offs) => branch(f, "bpl", offs),
        Operation::Jump(Minus(true), offs) => branch(f, "bmi", offs),
        Operation::Jump(Carry(false), offs) => branch(f, "bcc", offs),
        Operation::Jump(Carry(true), offs) => branch(f, "bcs", offs),
        Operation::Jump(True, offs) => branch(f, "jmp", offs),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn random_source() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn rmw_dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    fn opcode(oper: MonadicOperation) -> &'static str {
        match oper {
            Decrement => "dec",
            Increment => "inc",
            LeftShiftArithmetic => "asl",
            RightShiftLogical => "lsr",
            RotateRightThruCarry => "ror",
            RotateLeftThruCarry => "rol",
            _ => panic!("I don't know the opcode for {:?}", oper),
        }
    }

    match op {
        Operation::Monadic(Width::Width8, Decrement, X, X) => {
            write!(f, "\tdex")
        }
        Operation::Monadic(Width::Width8, Decrement, Y, Y) => {
            write!(f, "\tdey")
        }
        Operation::Monadic(Width::Width8, Increment, X, X) => {
            write!(f, "\tinx")
        }
        Operation::Monadic(Width::Width8, Increment, Y, Y) => {
            write!(f, "\tiny")
        }
        Operation::Monadic(Width::Width8, oper, d, _) => syn(f, opcode(oper), d),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn rmw_op(cmos: bool) -> Operation {
    // Operations which can be performed on either memory or the accumulator
    let ma = vec![
        LeftShiftArithmetic,
        RightShiftLogical,
        RotateLeftThruCarry,
        RotateRightThruCarry,
    ];

    // Operations which can be performed on either memory or an index register
    let mxy = vec![Decrement, Increment];

    fn xy(cmos: bool) -> Datum {
        // Pick an index register (if CMOS this includes the accumulator)
        if cmos {
            *[X, Y, A].choose(&mut rand::thread_rng()).unwrap()
        } else {
            *[X, Y].choose(&mut rand::thread_rng()).unwrap()
        }
    }

    if random() {
        let op = *ma.choose(&mut rand::thread_rng()).unwrap();
        let d = if random() { A } else { random_absolute() };
        Operation::Monadic(Width::Width8, op, d, d)
    } else {
        let op = *mxy.choose(&mut rand::thread_rng()).unwrap();
        let d = if random() {
            xy(cmos)
        } else {
            random_absolute()
        };
        Operation::Monadic(Width::Width8, op, d, d)
    }
}

fn alu_6502() -> Operation {
    // randomly generate the instructions ora, and, eor, adc, sbc, cmp
    // these all have the same available addressing modes
    let ops = vec![AddWithCarry, And, Or, ExclusiveOr, SubtractWithCarry];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();
    Operation::Dyadic(Width::Width8, op, A, random_source(), A)
}

fn transfers_6502() -> Operation {
    let reg = if random() { X } else { Y };
    if random() {
        Operation::Move(A, reg)
    } else {
        Operation::Move(reg, A)
    }
}

fn loads() -> Operation {
    let reg = randomly!( { A } { X } { Y } );

    if random() {
        Operation::Move(random_absolute(), reg)
    } else {
        Operation::Move(random_immediate(), reg)
    }
}

fn stores() -> Operation {
    let reg = randomly!( { A } { X } { Y } );

    Operation::Move(reg, random_absolute())
}

fn secl_6502() -> Operation {
    randomly!(
        {Operation::Carry(random())}
        {Operation::Decimal(random())}
        {Operation::Overflow(false)}
    )
}

fn branches() -> Operation {
    fn j() -> FlowControl {
        if random() {
            FlowControl::Forward(rand::thread_rng().gen_range(1..3))
        } else {
            FlowControl::Backward(rand::thread_rng().gen_range(1..3))
        }
    }

    fn cond() -> Test {
        randomly!(
           { Test::True}
           { Test::Minus(random())}
           { Test::Zero(random())}
           { Test::Overflow(random())}
           { Test::Carry(random())})
    }

    Operation::Jump(cond(), j())
}

fn compares() -> Operation {
    randomly!(
    { Operation::BitCompare(random_absolute(), A)}
    { Operation::Dyadic(Width::Width8, Subtract, A, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, X, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, Y, random_source(), Datum::Zero)}
    )
}

fn reg_mos6502(name: &str) -> Result<Datum, &'static str> {
    if name == "a" {
        return Ok(Datum::Register(R::A));
    }
    if name == "x" {
        return Ok(Datum::Register(R::Xl));
    }
    if name == "y" {
        return Ok(Datum::Register(R::Yl));
    }
    reg_by_name(name)
}

pub const MOS65C02: Machine = Machine {
    name: "65c02",
    reg_by_name: reg_mos6502,
};

pub const MOS6502: Machine = Machine {
    name: "6502",
    reg_by_name: reg_mos6502,
};

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn reg_names() {
        assert_eq!(reg_mos6502("a").unwrap(), A);
        assert_eq!(reg_mos6502("x").unwrap(), X);
        assert_eq!(reg_mos6502("y").unwrap(), Y);
        assert_eq!(reg_mos6502("m6").unwrap(), Datum::Absolute(6));
        assert!(reg_mos6502("n").is_err());
        assert!(reg_mos6502("m").is_err());
    }

    use crate::machine::mos6502::{A, X, Y};
    use crate::machine::DyadicOperation;
    use crate::machine::Operation;
    use crate::Datum;

    extern crate mos6502;
    use mos6502::address::Address;
    use mos6502::cpu;
    use mos6502::registers::Status;
    use rand::random;

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
}
