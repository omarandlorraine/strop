use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::reg_by_name;
use crate::machine::standard_implementation;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{AddWithCarry, And, ExclusiveOr, Or};
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Width;
use crate::machine::R;

use rand::random;
use strop::randomly;

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
        Operation::Shift(ShiftType::RightArithmetic, thing) => syn(f, "lsr", thing),
        Operation::Shift(ShiftType::LeftArithmetic, thing) => syn(f, "asl", thing),
        Operation::Shift(ShiftType::RightRotateThroughCarry, thing) => syn(f, "ror", thing),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, thing) => syn(f, "rol", thing),
        Operation::Dyadic(Width::Width8, AddWithCarry, A, thing, A) => syn(f, "adc", thing),
        Operation::Dyadic(Width::Width8, And, A, thing, A) => syn(f, "and", thing),
        Operation::Dyadic(Width::Width8, ExclusiveOr, A, thing, A) => syn(f, "eor", thing),
        Operation::Dyadic(Width::Width8, Or, A, thing, A) => syn(f, "ora", thing),
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
        Operation::Carry(false) => write!(f, "\tclc"),
        Operation::Carry(true) => write!(f, "\tsec"),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

pub fn instr_length_6502(insn: &Instruction) -> usize {
    fn length(dat: Datum) -> usize {
        match dat {
            Datum::Register(_) => 1,
            Datum::Imm8(_) => 2,
            Datum::Absolute(addr) => {
                if addr < 256 {
                    2
                } else {
                    3
                }
            }
            _ => 0,
        }
    }

    match insn.operation {
        Operation::Move(Datum::Register(_), Datum::Register(_)) => 1,
        Operation::Move(Datum::Register(_), dat) => length(dat),
        Operation::Move(dat, Datum::Register(_)) => length(dat),
        Operation::Shift(_, dat) => length(dat),
        Operation::Monadic(Width::Width8, MonadicOperation::Increment, dat, _) => length(dat),
        Operation::Monadic(Width::Width8, MonadicOperation::Decrement, dat, _) => length(dat),
        Operation::Dyadic(Width::Width8, _, _, dat, _) => length(dat),
        Operation::Carry(_) => 1,
        _ => 0,
    }
}

fn random_source() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn random_xy() -> Datum {
    randomly!(
        { Datum::Register(R::Xl)}
        { Datum::Register(R::Yl)}
    )
}

fn random_axy() -> Datum {
    randomly!(
        { Datum::Register(R::Xl)}
        { Datum::Register(R::Yl)}
        { Datum::Register(R::A)}
    )
}

fn incdec_axy() -> Operation {
    // the CMOS varieties have inc and dec for accumulator
    let reg = random_axy();
    if random() {
        Operation::Monadic(Width::Width8, MonadicOperation::Decrement, reg, reg)
    } else {
        Operation::Monadic(Width::Width8, MonadicOperation::Increment, reg, reg)
    }
}

fn incdec_xy() -> Operation {
    // earlier 6502s can increment and decrement X and Y only.
    let reg = random_xy();
    if random() {
        Operation::Monadic(Width::Width8, MonadicOperation::Decrement, reg, reg)
    } else {
        Operation::Monadic(Width::Width8, MonadicOperation::Increment, reg, reg)
    }
}

fn alu_6502() -> Operation {
    // randomly generate the instructions ora, and, eor, adc, sbc, cmp
    // these all have the same available addressing modes
    let ops = vec![AddWithCarry, And, Or, ExclusiveOr];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();
    Operation::Dyadic(Width::Width8, op, A, random_source(), A)
}

fn transfers_6502() -> Operation {
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

fn loadstore_6502() -> Operation {
    let addr = random_absolute();

    let reg =
        randomly!( { Datum::Register(R::A)} { Datum::Register(R::Xl)} { Datum::Register(R::Yl)} );

    if random() && reg != Datum::Zero {
        Operation::Move(addr, reg)
    } else {
        Operation::Move(reg, addr)
    }
}

fn secl_6502() -> Operation {
    Operation::Carry(random())
}

fn shifts_6502() -> Operation {
    let sht = randomly!(
        { ShiftType::LeftArithmetic}
        { ShiftType::RightArithmetic}
        { ShiftType::LeftRotateThroughCarry}
        { ShiftType::RightRotateThroughCarry}
    );
    let dat = if random() {
        Datum::Register(R::A)
    } else {
        random_absolute()
    };
    Operation::Shift(sht, dat)
}

const ALU_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: alu_6502,
};

const FLAG_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: secl_6502,
};

const NMOS6502_INSTRUCTIONS: [Instruction; 2] = [ALU_INSTRUCTIONS, FLAG_INSTRUCTIONS];

const CMOS6502_INSTRUCTIONS: [Instruction; 2] = [ALU_INSTRUCTIONS, FLAG_INSTRUCTIONS];

pub fn random_insn_65c02() -> Instruction {
    let mut op = *CMOS6502_INSTRUCTIONS
        .choose(&mut rand::thread_rng())
        .unwrap();
    op.randomize();
    op
}

fn random_insn_6502() -> Instruction {
    let mut op = *NMOS6502_INSTRUCTIONS
        .choose(&mut rand::thread_rng())
        .unwrap();
    op.randomize();
    op
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
    random_insn: random_insn_65c02,
    reg_by_name: reg_mos6502,
};

pub const MOS6502: Machine = Machine {
    name: "6502",
    random_insn: random_insn_6502,
    reg_by_name: reg_mos6502,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn find_it(opcode: &'static str, insn: &Instruction) {
        let mut i = insn.clone();
        let mut found_it = false;

        for _i in 0..5000 {
            i.randomize();
            let d = format!("{}", i);

            // Does the disassembler output something starting with a tab?
            assert!(
                d[0..1] == "\t".to_owned(),
                "Cannot disassemble {:?}, got {}",
                i.operation,
                d
            );

            // Is the opcode a substring of whatever the disassembler spat out?
            found_it |= d.contains(opcode);

            // Does this instruction have a length
            assert!(i.len() > 0, "No instruction length for {}", i);
        }
        assert!(found_it, "Couldn't find instruction {}", opcode);
    }

    #[test]
    fn reg_names() {
        assert_eq!(reg_mos6502("a").unwrap(), A);
        assert_eq!(reg_mos6502("x").unwrap(), X);
        assert_eq!(reg_mos6502("y").unwrap(), Y);
        assert_eq!(reg_mos6502("m6").unwrap(), Datum::Absolute(6));
        assert!(reg_mos6502("n").is_err());
        assert!(reg_mos6502("m").is_err());
    }

    #[test]
    fn instruction_set_6502() {
        find_it("adc", &ALU_INSTRUCTIONS);
        find_it("and", &ALU_INSTRUCTIONS);
        find_it("asl", &ALU_INSTRUCTIONS);
        find_it("bit", &ALU_INSTRUCTIONS);
        find_it("bcc", &ALU_INSTRUCTIONS);
        find_it("bcs", &ALU_INSTRUCTIONS);
        find_it("beq", &ALU_INSTRUCTIONS);
        find_it("bmi", &ALU_INSTRUCTIONS);
        find_it("bne", &ALU_INSTRUCTIONS);
        find_it("bpl", &ALU_INSTRUCTIONS);
        // not bothering with brk; it's some kind of buggy software interrupt instruction.
        find_it("bvc", &ALU_INSTRUCTIONS);
        find_it("bvs", &ALU_INSTRUCTIONS);
        find_it("clc", &FLAG_INSTRUCTIONS);
        find_it("cld", &FLAG_INSTRUCTIONS);
        // not bothering with cli; strop does not handle interrupts
        find_it("clv", &FLAG_INSTRUCTIONS);
        find_it("cmp", &ALU_INSTRUCTIONS);
        find_it("cpx", &ALU_INSTRUCTIONS);
        find_it("cpy", &ALU_INSTRUCTIONS);
        find_it("dec", &ALU_INSTRUCTIONS);
        find_it("dex", &ALU_INSTRUCTIONS);
        find_it("dey", &ALU_INSTRUCTIONS);
        find_it("eor", &ALU_INSTRUCTIONS);
        find_it("inc", &ALU_INSTRUCTIONS);
        find_it("inx", &ALU_INSTRUCTIONS);
        find_it("iny", &ALU_INSTRUCTIONS);
        find_it("jmp", &ALU_INSTRUCTIONS);
        // not bothering with jsr; strop does not call subroutines
        find_it("lda", &ALU_INSTRUCTIONS);
        find_it("ldx", &ALU_INSTRUCTIONS);
        find_it("ldy", &ALU_INSTRUCTIONS);
        find_it("lsr", &ALU_INSTRUCTIONS);
        // not bothering with nop; there's NO Point
        find_it("rol", &ALU_INSTRUCTIONS);
        find_it("ror", &ALU_INSTRUCTIONS);
        // not bothering with rti; strop does not handle interrupts
        // not bothering with rts; strop does not call subroutines
        // not bothering with sei; strop does not handle interrupts
        find_it("sbc", &ALU_INSTRUCTIONS);
        find_it("sta", &ALU_INSTRUCTIONS);
        find_it("sty", &ALU_INSTRUCTIONS);
        // as for txs tsx pha pla php plp, we need ot figure out how/if we're going to implement a stack.
        find_it("stx", &ALU_INSTRUCTIONS);
        find_it("stx", &ALU_INSTRUCTIONS);
        find_it("sec", &FLAG_INSTRUCTIONS);
        find_it("sed", &FLAG_INSTRUCTIONS);
        find_it("tax", &ALU_INSTRUCTIONS);
        find_it("txa", &ALU_INSTRUCTIONS);
        find_it("tay", &ALU_INSTRUCTIONS);
        find_it("tya", &ALU_INSTRUCTIONS);
    }
}
