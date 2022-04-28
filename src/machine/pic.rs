use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Datum;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::Operation;
use crate::machine::PicVariant;
use crate::machine::ShiftType;
use crate::machine::R;

use rand::random;
use strop::randomly;

fn dasm(op: Operation, fr: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match op {
        Operation::Add(Datum::Register(R::A), Datum::Absolute(f), false) => {
            write!(fr, "\taddwf {}, 1", f)
        }
        Operation::Add(Datum::Absolute(f), Datum::Register(R::A), false) => {
            write!(fr, "\taddwf {}, 0", f)
        }
        Operation::Add(Datum::Imm8(k), Datum::Register(R::A), false) => {
            write!(fr, "\taddlw {}, 0", k)
        }
        Operation::And(Datum::Imm8(k), Datum::Register(R::A)) => {
            write!(fr, "\tandlw {}, 0", k)
        }
        Operation::And(Datum::Absolute(f), Datum::Register(R::A)) => {
            write!(fr, "\tandwf {}, 0", f)
        }
        Operation::And(Datum::Register(R::A), Datum::Absolute(f)) => {
            write!(fr, "\tandwf {}, 1", f)
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
        Operation::Increment(Datum::Absolute(f)) => {
            write!(fr, "\tincf {}, 1", f)
        }
        Operation::Decrement(Datum::Absolute(f)) => {
            write!(fr, "\tdecf {}, 1", f)
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
    let w = Datum::Register(R::A);
    randomly!(
        { Operation::And(random_immediate(), w)} // andlw
        { Operation::And(random_absolute(), w)}  // andwf something, 0
        { Operation::And(w, random_absolute())}  // andwf something, 1
    )
}

fn store_pic(_mach: Machine) -> Operation {
    // TODO: There also is movf f,d, which just updates the Z flag
    randomly!(
        { Operation::Move(Datum::Zero, random_accumulator_or_absolute())} // clrw and clrf f
            { Operation::Move(random_absolute(), Datum::Register(R::A))}      // movf f
                { Operation::Move(random_immediate(), Datum::Register(R::A))}     // movlw k
                    { Operation::Move(Datum::Register(R::A), random_absolute())}      // movwf f
    )
}

pub fn instr_pic(mach: Machine) -> Instruction {
    randomly!(
        { Instruction::new(mach, shifts_pic, dasm)}
        { Instruction::new(mach, and_pic, dasm)}
        { Instruction::new(mach, add_pic, dasm)}
        { Instruction::new(mach, store_pic, dasm)}
        { Instruction::new(mach, inc_dec_pic, dasm)}
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclude_instructions() {
        // I've seen some instructions generated that are not part of any PIC's instruction set.
        // This test fails if it can get the same instruction again.

        fn check(_pic: PicVariant, fname: &'static str, op: Operation) {
            match op {
                Operation::Move(Datum::Register(R::A), Datum::Register(R::A)) => {
                    panic!("{} produced a move from W to W", fname)
                }
                Operation::Add(Datum::Absolute(_), Datum::Absolute(_), _) => panic!(
                    "{} produced an Add operation with two operands in memory",
                    fname
                ),
                Operation::And(Datum::Absolute(_), Datum::Absolute(_)) => panic!(
                    "{} produced an And operation with two operands in memory",
                    fname
                ),
                _ => {}
            }
        }

        fn excl(pic: PicVariant) {
            for _i in 0..50000 {
                check(pic, "store_pic", store_pic(Machine::Pic(pic)));
                check(pic, "and_pic", and_pic(Machine::Pic(pic)));
                check(pic, "shifts_pic", shifts_pic(Machine::Pic(pic)));
                check(pic, "inc_dec_pic", inc_dec_pic(Machine::Pic(pic)));
            }
            for _i in 0..5000 {
                let mut instr = instr_pic(Machine::Pic(PicVariant::Pic12));
                for _j in 0..500 {
                    instr.randomize();
                    check(pic, "something", instr.operation);
                }
            }
        }

        excl(PicVariant::Pic12);
    }

    fn find_it(opcode: &'static str, rnd: fn(Machine) -> Operation, mach: PicVariant) {
        for _i in 0..500 {
            let i = Instruction::new(Machine::Pic(mach), rnd, dasm);
            let d = format!("{}", i);
            if d.contains(opcode) {
                return;
            }
        }
        panic!("Couldn't find instruction {}", opcode);
    }

    #[test]
    fn instr_set_pic14() {
        // TODO: bcf bsf btfsc btfss comf decfsz incfsz iorlw iorwf subwf sublw swapf xorwf xorlw
        // I don't think we need to bother with call, clrwdt retfie, retlw, return, sleep, nop
        find_it("addwf", add_pic, PicVariant::Pic14);
        find_it("addlw", add_pic, PicVariant::Pic14);
        find_it("andwf", and_pic, PicVariant::Pic14);
        find_it("andlw", and_pic, PicVariant::Pic14);
        find_it("clrf", store_pic, PicVariant::Pic14);
        find_it("clrw", store_pic, PicVariant::Pic14);
        find_it("decf", inc_dec_pic, PicVariant::Pic14);
        find_it("incf", inc_dec_pic, PicVariant::Pic14);
        find_it("movf", store_pic, PicVariant::Pic14);
        find_it("movlw", store_pic, PicVariant::Pic14);
        find_it("movwf", store_pic, PicVariant::Pic14);
        find_it("rlf", shifts_pic, PicVariant::Pic14);
        find_it("rrf", shifts_pic, PicVariant::Pic14);
    }
}
