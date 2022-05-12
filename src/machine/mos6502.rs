use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_absolute;
use crate::machine::random_immediate;
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

pub fn random_insn_65c02() -> Instruction {
    randomly!(
        { Instruction::new(incdec_axy, dasm, instr_length_6502)}
        { Instruction::new(alu_6502, dasm, instr_length_6502)}
        { Instruction::new(transfers_6502, dasm, instr_length_6502)}
        { Instruction::new(shifts_6502, dasm, instr_length_6502)}
        { Instruction::new(loadstore_6502, dasm, instr_length_6502)}
        { Instruction::new(secl_6502, dasm, instr_length_6502)}
    )
}

fn random_insn_6502() -> Instruction {
    randomly!(
        { Instruction::new(incdec_xy, dasm, instr_length_6502)}
        { Instruction::new(alu_6502, dasm, instr_length_6502)}
        { Instruction::new(transfers_6502, dasm, instr_length_6502)}
        { Instruction::new(shifts_6502, dasm, instr_length_6502)}
        { Instruction::new(loadstore_6502, dasm, instr_length_6502)}
        { Instruction::new(secl_6502, dasm, instr_length_6502)}
    )
}

pub fn reg_by_name(name: &str) -> Datum {
    if name == "a" {
        return Datum::Register(R::A);
    }
    if name == "x" {
        return Datum::Register(R::Xl);
    }
    if name == "y" {
        return Datum::Register(R::Yl);
    }
    todo!();
}

pub const Mos65c02: Machine = Machine {
    id: 0,
    name: "65c02",
    description: "The CMOS 6502 variant, including new instructions like phx and stz",
    random_insn: random_insn_6502,
    reg_by_name,
};

pub const Mos6502: Machine = Machine {
    id: 0,
    name: "6502",
    description: "A generic 6502",
    random_insn: random_insn_6502,
    reg_by_name,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::tests::disasm;

    fn find_it(opcode: &'static str, rnd: fn(Machine) -> Operation, mach: Mos6502Variant) {
        for _i in 0..5000 {
            let i = Instruction::new(Machine::Mos6502(mach), rnd, dasm);
            let d = format!("{}", i);
            if d.contains(opcode) {
                return;
            }
        }
        panic!("Couldn't find instruction {}", opcode);
    }

    fn core_instruction_set(mach: Mos6502Variant) {
        find_it("adc", alu_6502, mach);
        find_it("and", alu_6502, mach);
        find_it("asl", shifts_6502, mach);
        // TODO: bcc bcs beq bit bmi bne bpl bvc bvs cld clv dec inc jmp ora pha pla sbc sed tsx txs
        // I don't think we need to bother with brk cli jsr nop php plp rti rts sei
        find_it("clc", secl_6502, mach);
        // Temporarily removed cmp cpx cpy untill I figure out what I want to do with them
        find_it("dex", incdec_6502, mach);
        find_it("dey", incdec_6502, mach);
        find_it("eor", alu_6502, mach);
        find_it("inx", incdec_6502, mach);
        find_it("iny", incdec_6502, mach);
        find_it("lda", loadstore_6502, mach);
        find_it("ldx", loadstore_6502, mach);
        find_it("ldy", loadstore_6502, mach);
        find_it("lsr", shifts_6502, mach);
        find_it("rol", shifts_6502, mach);
        find_it("ror", shifts_6502, mach);
        find_it("sec", secl_6502, mach);
        find_it("sta", loadstore_6502, mach);
        find_it("stx", loadstore_6502, mach);
        find_it("sty", loadstore_6502, mach);
        find_it("tax", transfers_6502, mach);
        find_it("tay", transfers_6502, mach);
        find_it("txa", transfers_6502, mach);
        find_it("tya", transfers_6502, mach);
    }

    #[test]
    fn instruction_set_65c02() {
        core_instruction_set(Mos6502Variant::Cmos);
        // TODO: bra phx plx phy ply trb tsb bbr bbs rmb smb
        find_it("dea", incdec_6502, Mos6502Variant::Cmos);
        find_it("dea", incdec_6502, Mos6502Variant::Cmos);
        find_it("ina", incdec_6502, Mos6502Variant::Cmos);
        find_it("ina", incdec_6502, Mos6502Variant::Cmos);
        find_it("stz", loadstore_6502, Mos6502Variant::Cmos);
    }

    #[test]
    fn instruction_set_6502() {
        core_instruction_set(Mos6502Variant::Nmos);
    }

    #[test]
    fn instruction_set_65i02() {
        core_instruction_set(Mos6502Variant::IllegalInstructions);
    }

    fn instrlen(m: Machine) {
        for _i in 0..5000 {
            let instr = instr_6502(m);
            let len = instr_length_6502(instr.operation);
            if !(len > 0) {
                println!("unknown length for instruction: {}", instr);
                panic!();
            }
        }
    }

    #[test]
    fn instruction_lengths_6502() {
        instrlen(Machine::Mos6502(Mos6502Variant::Nmos));
    }

    #[test]
    fn disassembler() {
        disasm(Machine::Mos6502(Mos6502Variant::Nmos));
        disasm(Machine::Mos6502(Mos6502Variant::Cmos));
        disasm(Machine::Mos6502(Mos6502Variant::Ricoh2a03));
        disasm(Machine::Mos6502(Mos6502Variant::IllegalInstructions));
    }
}
