use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Operation;
use crate::machine::PicVariant;
use crate::machine::ShiftType;
use crate::machine::R;

use crate::machine::rand::Rng;
use rand::random;

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", op)
}

fn random_accumulator_or_absolute() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else {
        random_absolute()
    }
}

fn inc_dec_pic(_mach: Machine) -> Operation {
    // TODO: These instructions can optionally write to W instead of the F.
    if random() {
        Operation::Increment(random_absolute()) // incf f
    } else {
        Operation::Decrement(random_absolute()) // decf f
    }
}

fn add_pic(mach: Machine) -> Operation {
    let dst = random_accumulator_or_absolute();
    if dst == Datum::Register(R::A) && mach != Machine::Pic(PicVariant::Pic12) && random() {
        // This is an immediate add. Not available on PIC12.
        Operation::Add(random_immediate(), Datum::Register(R::A), false) // addlw k
    } else if random() {
        Operation::Add(random_absolute(), Datum::Register(R::A), false) // addwf f
    } else {
        Operation::Add(Datum::Register(R::A), random_absolute(), false) // addwf f,d
    }
}

fn shifts_pic(_mach: Machine) -> Operation {
    // TODO: These instructions can optionally write to W instead of the F.
    let shtype = if random() {
        ShiftType::RightRotateThroughCarry
    } else {
        ShiftType::LeftRotateThroughCarry
    };
    Operation::Shift(shtype, random_absolute()) // rlf f,d and rrf f,d
}

fn and_pic(_mach: Machine) -> Operation {
    let dst = random_accumulator_or_absolute();
    if dst == Datum::Register(R::A) && random() {
        // andlw
        Operation::And(random_immediate(), dst)
    } else if random() {
        Operation::And(random_absolute(), dst)
    } else {
        Operation::And(dst, random_absolute())
    }
}

fn store_pic(_mach: Machine) -> Operation {
    // TODO: There also is movf f,d, which just updates the Z flag
    match rand::thread_rng().gen_range(0, 4) {
        0 => Operation::Move(Datum::Zero, random_accumulator_or_absolute()), // clrw and clrf f
        1 => Operation::Move(random_accumulator_or_absolute(), Datum::Register(R::A)), // movf f
        2 => Operation::Move(random_immediate(), Datum::Register(R::A)),     // movlw k
        _ => Operation::Move(Datum::Register(R::A), random_accumulator_or_absolute()), // movwf f
    }
}

pub fn instr_pic(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 5) {
        0 => Instruction::new(mach, shifts_pic, dasm),
        1 => Instruction::new(mach, and_pic, dasm),
        2 => Instruction::new(mach, add_pic, dasm),
        3 => Instruction::new(mach, store_pic, dasm),
        _ => Instruction::new(mach, inc_dec_pic, dasm),
    }
    // TODO: Add the following other instructions:
    // bcf bsf btfsc btfss (call) (clrwdt) comf decfsz (goto) incfsz iorlw iorwf (nop) (option) (retlw) (sleep) subwf swapf (tris) xorlw xorwf
}
