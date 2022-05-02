use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::DyadicOperation::{Add, AddWithCarry, And, ExclusiveOr, Or};
use crate::machine::FlowControl;
use crate::machine::Instruction;
use crate::machine::MonadicOperation::{Complement, Decrement, Increment, Negate};
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::Test;
use crate::machine::Width;
use crate::machine::R;
use crate::Datum;
use crate::Machine;

use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
use rand::random;
use strop::randomly;

const A: Datum = Datum::Register(R::A);
const XL: Datum = Datum::Register(R::Xl);
const YL: Datum = Datum::Register(R::Yl);
const XH: Datum = Datum::Register(R::Xh);
const YH: Datum = Datum::Register(R::Yh);
const X: Datum = Datum::RegisterPair(R::Xh, R::Xl);
const Y: Datum = Datum::RegisterPair(R::Yh, R::Yl);

fn random_stm8_operand() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn random_register() -> Datum {
    let regs = vec![A, X, Y];
    *regs.choose(&mut rand::thread_rng()).unwrap()
}

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn bit(
        f: &mut std::fmt::Formatter,
        s: &'static str,
        d: u16,
        bitnumber: u8,
    ) -> std::fmt::Result {
        write!(f, "\t{}, ${:4}, #{}", s, d, bitnumber)
    }
    fn syn(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        match d {
            Datum::Imm8(val) => write!(f, "\t{} #${:2}", s, val),
            Datum::Absolute(addr) if addr < 256 => write!(f, "\t{} ${:2}", s, addr),
            Datum::Absolute(addr) => write!(f, "\t{} ${:4}", s, addr),
            A => write!(f, "\t{} a", s),
            X => write!(f, "\t{}w x", s),
            Y => write!(f, "\t{}w y", s),
            _ => write!(f, "{} {:?}", s, d),
        }
    }

    fn dsyn(f: &mut std::fmt::Formatter, s: &'static str, r: Datum, d: Datum) -> std::fmt::Result {
        let (suffix, regname) = match r {
            A => ("", "a"),
            X => ("w", "x"),
            Y => ("w", "y"),
            _ => panic!("dsyn baulks at {:?} for r", r),
        };

        match d {
            Datum::Imm8(val) => write!(f, "\t{}{} {}, #${:4}", s, suffix, regname, val),
            Datum::Absolute(addr) if addr < 256 => {
                write!(f, "\t{}{} {}, ${:2}", s, suffix, regname, addr)
            }
            Datum::Absolute(addr) => write!(f, "\t{}{} {}, ${:4}", s, suffix, regname, addr),
            A => write!(f, "\t{}{} {}, a", suffix, regname, s),
            _ => write!(f, "{}{} {}, {:?}", s, suffix, regname, d),
        }
    }

    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::Xh => "xh",
            R::Xl => "xl",
            R::Yh => "yh",
            R::Yl => "yl",
            _ => panic!(),
        }
    }

    fn distest(test: Test) -> &'static str {
        match test {
            Test::Carry(true) => "jrc",
            Test::Carry(false) => "jrnc",
            Test::True => "jr",
            _ => panic!(),
        }
    }

    fn btjt(
        f: &mut std::fmt::Formatter,
        addr: u16,
        bit_no: u8,
        v: bool,
        target: FlowControl,
    ) -> std::fmt::Result {
        let op = if v { "btjt" } else { "btjf" };
        match target {
            FlowControl::Forward(offs) => write!(f, "\t{} {}, {}, +{}", op, addr, bit_no, offs),
            FlowControl::Backward(offs) => write!(f, "\t{} {}, {}, -{}", op, addr, bit_no, offs),
            FlowControl::Invalid => panic!(),
            FlowControl::FallThrough => panic!(),
        }
    }

    fn jump(f: &mut std::fmt::Formatter, s: &'static str, target: FlowControl) -> std::fmt::Result {
        match target {
            FlowControl::Forward(offs) => write!(f, "\t{} +{}", s, offs),
            FlowControl::Backward(offs) => write!(f, "\t{} -{}", s, offs),
            FlowControl::Invalid => panic!(),
            FlowControl::FallThrough => panic!(),
        }
    }

    match op {
        Operation::Compare(d, r) => dsyn(f, "cp", r, d),
        Operation::BitCompare(d, r) => dsyn(f, "bcp", r, d),
        Operation::Dyadic(_, And, _, d, r) => dsyn(f, "and", r, d),
        Operation::Dyadic(_, Add, _, d, r) => dsyn(f, "add", r, d),
        Operation::Dyadic(_, AddWithCarry, _, d, r) => dsyn(f, "adc", r, d),
        Operation::Dyadic(_, Or, _, d, r) => dsyn(f, "or", r, d),
        Operation::Dyadic(_, ExclusiveOr, _, d, r) => dsyn(f, "xor", r, d),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, d) => syn(f, "rlc", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => syn(f, "rrc", d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => syn(f, "sla", d),
        Operation::Shift(ShiftType::RightArithmetic, d) => syn(f, "sra", d),
        Operation::Move(Datum::Zero, r) => syn(f, "clr", r),
        Operation::Monadic(_, Decrement, _, r) => syn(f, "dec", r),
        Operation::Monadic(_, Increment, _, r) => syn(f, "inc", r),
        Operation::Monadic(_, Complement, _, r) => syn(f, "cpl", r),
        Operation::Monadic(_, Negate, _, r) => syn(f, "neg", r),
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\tld {}, {}", regname(to), regname(from))
        }
        Operation::Move(Datum::Absolute(addr), Datum::Register(to)) => {
            write!(f, "\tld {}, {}", regname(to), addr)
        }
        Operation::Move(Datum::Register(to), Datum::Absolute(addr)) => {
            write!(f, "\tld {}, {}", addr, regname(to))
        }
        Operation::Move(Datum::Imm8(val), Datum::Register(to)) => {
            write!(f, "\tld #{}, {}", val, regname(to))
        }
        Operation::BitClear(Datum::Absolute(addr), bitnumber) => bit(f, "bres", addr, bitnumber),
        Operation::BitSet(Datum::Absolute(addr), bitnumber) => bit(f, "bset", addr, bitnumber),
        Operation::BitComplement(Datum::Absolute(addr), bitnumber) => {
            bit(f, "bcpl", addr, bitnumber)
        }
        Operation::BitCopyCarry(Datum::Absolute(addr), bitnumber) => {
            bit(f, "bccm", addr, bitnumber)
        }
        Operation::Carry(false) => write!(f, "\trcf"),
        Operation::Carry(true) => write!(f, "\tscf"),
        Operation::ComplementCarry => write!(f, "\tccf"),
        Operation::Jump(Test::Bit(addr, bit_no, t), target) => btjt(f, addr, bit_no, t, target),
        Operation::Jump(test, target) => jump(f, distest(test), target),
        _ => write!(f, "{:?}", op),
    }
}

fn clear(_mach: Machine) -> Operation {
    if random() {
        Operation::Move(Datum::Zero, random_register())
    } else {
        Operation::Move(Datum::Zero, random_absolute())
    }
}

fn twoargs(_mach: Machine) -> Operation {
    fn op(w: Width, a: Datum) -> Operation {
        let vs = vec![Add, AddWithCarry];
        let o = vs.choose(&mut rand::thread_rng());
        Operation::Dyadic(w, *o.unwrap(), a, random_absolute(), a)
    }

    if random() {
        op(Width::Width8, A)
    } else {
        let a = if random() { X } else { Y };

        op(Width::Width16, a)
    }
}

fn bits(_mach: Machine) -> Operation {
    let addr = random_absolute();
    let bit: u8 = rand::thread_rng().gen_range(0..=7);

    // the eight-bit diadic operations like and, xor, or, etc
    randomly!(
        { Operation::BitSet(addr, bit)}
        { Operation::BitClear(addr, bit)}
        { Operation::BitComplement(addr, bit)}
        { Operation::BitCopyCarry(addr, bit)}
    )
}

fn alu8(_mach: Machine) -> Operation {
    let ops = vec![And, Or, ExclusiveOr];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();
    Operation::Dyadic(Width::Width8, op, random_stm8_operand(), A, A)
}

fn shifts(_mach: Machine) -> Operation {
    let sht = randomly!(
        { ShiftType::LeftArithmetic}
        { ShiftType::RightArithmetic}
        { ShiftType::RightRotateThroughCarry}
        { ShiftType::LeftRotateThroughCarry}
    );

    let operand = if random() {
        random_absolute()
    } else {
        random_register()
    };

    Operation::Shift(sht, operand)
}

fn carry(_mach: Machine) -> Operation {
    randomly!(
        { Operation::Carry(false)}
        { Operation::Carry(true)}
        { Operation::ComplementCarry}
    )
}

fn compare(_mach: Machine) -> Operation {
    randomly!(
        { Operation::Compare(random_stm8_operand(), A)}
        { Operation::Compare(random_stm8_operand(), X)}
        { Operation::Compare(random_stm8_operand(), Y)}
        { Operation::BitCompare(random_stm8_operand(), A)}
    )
}

fn transfers(_mach: Machine) -> Operation {
    fn rando() -> Datum {
        let regs = vec![A, XL, XH, YL, YH];
        *regs.choose(&mut rand::thread_rng()).unwrap()
    }
    Operation::Move(rando(), rando())
}

pub fn jumps(_mach: Machine) -> Operation {
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
        { Test::Carry(random())}
        { Test::Bit(random(), rand::thread_rng().gen_range(0..7), random())}
        )
    }

    Operation::Jump(cond(), j())
}

fn oneargs(_mach: Machine) -> Operation {
    fn op(w: Width, a: Datum) -> Operation {
        let vs = vec![Complement, Negate, Increment, Decrement];
        let o = vs.choose(&mut rand::thread_rng());
        Operation::Monadic(w, *o.unwrap(), a, a)
    }

    if random() {
        let a = if random() { A } else { random_immediate() };

        op(Width::Width8, a)
    } else {
        let a = if random() { X } else { Y };

        op(Width::Width16, a)
    }
}

pub fn instr_stm8(mach: Machine) -> Instruction {
    randomly!(
    { Instruction::new(mach, clear, dasm)}
    { Instruction::new(mach, transfers, dasm)}
    { Instruction::new(mach, alu8, dasm)}
    { Instruction::new(mach, bits, dasm)}
    { Instruction::new(mach, carry, dasm)}
    { Instruction::new(mach, compare, dasm)}
    { Instruction::new(mach, jumps, dasm)}
    { Instruction::new(mach, oneargs, dasm)}
    { Instruction::new(mach, twoargs, dasm)}
    { Instruction::new(mach, shifts, dasm)}
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::tests::disasm;

    fn find_it(opcode: &'static str, rnd: fn(Machine) -> Operation) {
        for _i in 0..5000 {
            let i = Instruction::new(Machine::Stm8, rnd, dasm);
            let d = format!("{}", i);
            if d.contains(opcode) {
                return;
            }
        }
        panic!("Couldn't find instruction {}", opcode);
    }

    #[test]
    fn disassembler() {
        crate::machine::tests::disasm(Machine::Stm8);
    }

    #[test]
    fn instruction_set_stm8() {
        // I don't think we need call callf callr halt iret jrf jrih jril jrm nop ret retf rim sim trap wfe wfi
        // TODO: div divw exg exgw ld ldw mov mul pop popw push pushw rvf sbc sub subw swap tnz tnzw
        // TODO: conditional jumps, relative jump, more shifts
        find_it("adc", twoargs);
        find_it("add", twoargs);
        find_it("addw", twoargs);
        find_it("and", alu8);
        find_it("bccm", bits);
        find_it("bcp", compare);
        find_it("btjf", jumps);
        find_it("btjt", jumps);
        find_it("bcpl", bits);
        find_it("bset", bits);
        find_it("bres", bits);
        find_it("cpl", oneargs);
        find_it("cplw", oneargs);
        find_it("cp", compare);
        find_it("cpw", compare);
        find_it("ccf", carry);
        find_it("clr", clear);
        find_it("clrw", clear);
        find_it("dec", oneargs);
        find_it("decw", oneargs);
        find_it("inc", oneargs);
        find_it("incw", oneargs);
        find_it("jrc", jumps);
        find_it("jrnc", jumps);
        find_it("ld a, xh", transfers);
        find_it("ld yl, a", transfers);
        find_it("neg", oneargs);
        find_it("negw", oneargs);
        find_it("or", alu8);
        find_it("rcf", carry);
        find_it("scf", carry);
        find_it("rlc", shifts);
        find_it("rlcw", shifts);
        find_it("rrc", shifts);
        find_it("rrcw", shifts);
        find_it("sla", shifts);
        find_it("slaw", shifts);
        find_it("xor", alu8);
    }
}
