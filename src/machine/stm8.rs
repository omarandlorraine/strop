use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::standard_implementation;
use crate::machine::DyadicOperation::{
    Add, AddWithCarry, And, Divide, ExclusiveOr, Multiply, Or, Subtract, SubtractWithBorrow,
};
use crate::machine::FlowControl;
use crate::machine::Instruction;
use crate::machine::MonadicOperation::{
    Complement, Decrement, Increment, LeftShiftArithmetic, Negate, RightShiftArithmetic,
    RotateLeftThruCarry, RotateRightThruCarry, Swap,
};
use crate::machine::Operation;
use crate::machine::Test;
use crate::machine::Width;
use crate::machine::R;
use crate::Datum;
use crate::Machine;
use crate::State;

use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
use crate::machine::reg_by_name;
use rand::random;
use std::convert::TryInto;
use strop::randomly;

const A: Datum = Datum::Register(R::A);
const XL: Datum = Datum::Register(R::Xl);
const YL: Datum = Datum::Register(R::Yl);
const XH: Datum = Datum::Register(R::Xh);
const YH: Datum = Datum::Register(R::Yh);
const X: Datum = Datum::RegisterPair(R::Xh, R::Xl);
const Y: Datum = Datum::RegisterPair(R::Yh, R::Yl);

fn random_imm16() -> Datum {
    let regs = vec![0, 1, 2, 3, 4, 5, 6];
    Datum::Imm16(*regs.choose(&mut rand::thread_rng()).unwrap())
}

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
            Test::Minus(true) => "jrmi",
            Test::Minus(false) => "jrpl",
            Test::Zero(true) => "jreq",
            Test::Zero(false) => "jrne",
            Test::Carry(true) => "jrc",
            Test::Carry(false) => "jrnc",
            Test::HalfCarry(true) => "jrh",
            Test::HalfCarry(false) => "jrnh",
            Test::Overflow(true) => "jrv",
            Test::Overflow(false) => "jrnv",
            Test::SignedLowerThanOrEqual => "jrsle",
            Test::SignedLowerThan => "jrslt",
            Test::SignedGreaterThanOrEqual => "jrsge",
            Test::SignedGreaterThan => "jrsgt",
            Test::UnsignedGreaterThan => "jrugt",
            Test::UnsignedLowerThanOrEqual => "jrule",
            Test::True => "jra",
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
        Operation::BitCompare(d, r) => dsyn(f, "bcp", r, d),
        Operation::Dyadic(_, And, _, d, r) => dsyn(f, "and", r, d),
        Operation::Dyadic(_, Add, _, d, r) => dsyn(f, "add", r, d),
        Operation::Dyadic(_, AddWithCarry, _, d, r) => dsyn(f, "adc", r, d),
        Operation::Dyadic(_, ExclusiveOr, _, d, r) => dsyn(f, "xor", r, d),
        Operation::Dyadic(_, Or, _, d, r) => dsyn(f, "or", r, d),
        Operation::Dyadic(_, SubtractWithBorrow, _, d, r) => dsyn(f, "sbc", r, d),
        Operation::Dyadic(_, Subtract, r, d, Datum::Zero) => dsyn(f, "cp", r, d),
        Operation::Dyadic(_, Subtract, _, d, r) => dsyn(f, "sub", r, d),
        Operation::Move(Datum::Zero, r) => syn(f, "clr", r),
        Operation::Monadic(_, Decrement, _, r) => syn(f, "dec", r),
        Operation::Monadic(_, Increment, _, r) => syn(f, "inc", r),
        Operation::Monadic(_, Complement, _, r) => syn(f, "cpl", r),
        Operation::Monadic(_, Negate, _, r) => syn(f, "neg", r),
        Operation::Monadic(_, Swap, _, r) => syn(f, "swap", r),
        Operation::Monadic(_, LeftShiftArithmetic, _, r) => syn(f, "sla", r),
        Operation::Monadic(_, RightShiftArithmetic, _, r) => syn(f, "sra", r),
        Operation::Monadic(_, RotateLeftThruCarry, _, r) => syn(f, "rlc", r),
        Operation::Monadic(_, RotateRightThruCarry, _, r) => syn(f, "rrc", r),
        Operation::Exchange(Width::Width16, X, Y) => {
            write!(f, "\texgw x, y")
        }
        Operation::Exchange(Width::Width8, Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\texg {}, {}", regname(to), regname(from))
        }
        Operation::Move(Datum::Register(from), Datum::Register(to)) => {
            write!(f, "\tld {}, {}", regname(to), regname(from))
        }
        Operation::Move(Datum::Absolute(addr), Datum::Register(to)) => {
            write!(f, "\tld {}, {}", regname(to), addr)
        }
        Operation::Move(Datum::Register(to), Datum::Absolute(addr)) => {
            write!(f, "\tld {}, {}", regname(to), addr)
        }
        Operation::Move(Datum::Imm8(val), Datum::Register(to)) => {
            write!(f, "\tld #{}, {}", regname(to), val)
        }
        Operation::Move(Datum::Imm16(val), X) => {
            write!(f, "\tldw x, #{}", val)
        }
        Operation::Move(Datum::Imm16(val), Y) => {
            write!(f, "\tldw y, #{}", val)
        }
        Operation::Move(Datum::Imm8(val), Datum::Absolute(to)) => {
            write!(f, "\tmov #{}, {}", val, to)
        }
        Operation::Move(Datum::Absolute(val), Datum::Absolute(to)) => {
            write!(f, "\tmov {}, {}", val, to)
        }
        Operation::BitClear(Datum::Absolute(addr), bitnumber) => bit(f, "bres", addr, bitnumber),
        Operation::BitSet(Datum::Absolute(addr), bitnumber) => bit(f, "bset", addr, bitnumber),
        Operation::BitComplement(Datum::Absolute(addr), bitnumber) => {
            bit(f, "bcpl", addr, bitnumber)
        }
        Operation::BitCopyCarry(Datum::Absolute(addr), bitnumber) => {
            bit(f, "bccm", addr, bitnumber)
        }
        Operation::Overflow(false) => write!(f, "\trvf"),
        Operation::Carry(false) => write!(f, "\trcf"),
        Operation::Carry(true) => write!(f, "\tscf"),
        Operation::ComplementCarry => write!(f, "\tccf"),
        Operation::Jump(Test::Bit(addr, bit_no, t), target) => btjt(f, addr, bit_no, t, target),
        Operation::Jump(test, target) => jump(f, distest(test), target),
        _ => write!(f, "{:?}", op),
    }
}

fn clear() -> Operation {
    if random() {
        Operation::Move(Datum::Zero, random_register())
    } else {
        Operation::Move(Datum::Zero, random_absolute())
    }
}

fn dasm_muldiv(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match op {
        Operation::Dyadic(Width::Width8, Multiply, A, X, X) => write!(f, "\tmul x, a"),
        Operation::Dyadic(Width::Width8, Multiply, A, Y, Y) => write!(f, "\tmul y, a"),
        Operation::Dyadic(Width::Width8, Divide, A, X, X) => write!(f, "\tdiv x, a"),
        Operation::Dyadic(Width::Width8, Divide, A, Y, Y) => write!(f, "\tdiv y, a"),
        Operation::Dyadic(Width::Width8, Divide, X, Y, Y) => write!(f, "\tdivw x, y"),
        _ => write!(f, "{:?}", op),
    }
}

fn instr_length_muldiv(insn: &Instruction) -> usize {
    match insn.operation {
        Operation::Dyadic(Width::Width8, Multiply, A, X, X) => 1,
        Operation::Dyadic(Width::Width8, Multiply, A, Y, Y) => 2,
        Operation::Dyadic(Width::Width8, Divide, A, X, X) => 1,
        Operation::Dyadic(Width::Width8, Divide, A, Y, Y) => 2,
        Operation::Dyadic(Width::Width8, Divide, X, Y, Y) => 1,
        _ => 0,
    }
}

fn impl_muldiv(insn: &Instruction, s: &mut State) -> FlowControl {
    fn div(s: &mut State, x: Datum, a: Datum) {
        let dividend = s.get_u16(x);
        let divisor = s.get_u8(a);

        if dividend.is_none() || divisor.is_none() {
            s.set_i8(A, None);
            s.set_i16(X, None);
            s.carry = None;
            return;
        }
        if divisor.unwrap() != 0 {
            let quotient: u16 = dividend.unwrap() / (divisor.unwrap() as u16);
            let remainder: u8 = (dividend.unwrap() % (divisor.unwrap() as u16))
                .try_into()
                .unwrap();
            s.set_u8(A, Some(remainder));
            s.set_u16(X, Some(quotient));
            s.carry = Some(false);
            s.zero = Some(quotient == 0);
        } else {
            // division by zero; the quotient and remainder are not written to the registers.
            s.carry = Some(true);
        }
    }

    fn divw(s: &mut State) {
        let dividend = s.get_u16(X);
        let divisor = s.get_u16(Y);

        if dividend.is_none() || divisor.is_none() {
            s.set_u16(X, None);
            s.set_u16(Y, None);
            s.carry = None;
            return;
        }
        if divisor.unwrap() == 0 {
            // division by zero; the quotient and remainder are indeterminate
            s.set_u16(X, None);
            s.set_u16(Y, None);
            s.carry = Some(false);
            s.zero = None;
        } else {
            let quotient: u16 = dividend.unwrap() / divisor.unwrap();
            let remainder: u16 = dividend.unwrap() % divisor.unwrap();
            s.set_u16(X, Some(quotient));
            s.set_u16(Y, Some(remainder));
            s.zero = Some(quotient == 0);
            s.carry = Some(true);
        }
    }

    fn mul(s: &mut State, a: Option<u8>, b: Option<u8>, dst: Datum) {
        if a.is_none() || b.is_none() {
            s.set_u8(dst, None);
            s.carry = None;
            return;
        }
        let product = (a.unwrap() as u16) * (b.unwrap() as u16);
        s.set_u16(dst, Some(product));
        s.carry = Some(false);
    }

    match insn.operation {
        Operation::Dyadic(Width::Width8, Multiply, A, X, X) => mul(s, s.get_u8(XL), s.get_u8(A), X),
        Operation::Dyadic(Width::Width8, Multiply, A, Y, Y) => mul(s, s.get_u8(YL), s.get_u8(A), Y),
        Operation::Dyadic(Width::Width8, Divide, A, X, X) => div(s, X, A),
        Operation::Dyadic(Width::Width8, Divide, A, Y, Y) => div(s, Y, A),
        Operation::Dyadic(Width::Width8, Divide, X, Y, Y) => divw(s),
        _ => unimplemented!(),
    }
    FlowControl::FallThrough
}

fn muldiv() -> Operation {
    randomly!(
        {Operation::Dyadic(Width::Width8, Multiply, A, X, X)}
        {Operation::Dyadic(Width::Width8, Multiply, A, Y, Y)}
        {Operation::Dyadic(Width::Width8, Divide, A, X, X)}
        {Operation::Dyadic(Width::Width8, Divide, A, Y, Y)}
        {Operation::Dyadic(Width::Width8, Divide, X, Y, Y)})
}

fn twoargs() -> Operation {
    fn op(w: Width, a: Datum) -> Operation {
        let vs = vec![Add, And, Or, Subtract];
        let o = vs.choose(&mut rand::thread_rng());
        Operation::Dyadic(w, *o.unwrap(), a, random_absolute(), a)
    }

    fn op8(w: Width, a: Datum) -> Operation {
        // These operations only work with 8-bit operands
        let vs = vec![AddWithCarry, ExclusiveOr, SubtractWithBorrow];
        let o = vs.choose(&mut rand::thread_rng());
        Operation::Dyadic(w, *o.unwrap(), a, random_absolute(), a)
    }

    randomly!(
    { op(Width::Width8, A) }
    { op8(Width::Width8, A) }
    { op(Width::Width16, X) }
    { op(Width::Width16, Y) }
    )
}

fn bits() -> Operation {
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

fn carry() -> Operation {
    randomly!(
        { Operation::Carry(false)}
        { Operation::Carry(true)}
        { Operation::ComplementCarry}
        { Operation::Overflow(false)}
    )
}

fn compare() -> Operation {
    randomly!(
    {Operation::BitCompare(random_stm8_operand(), A)}
    {Operation::Dyadic(Width::Width8, Subtract, A, random_stm8_operand(), Datum::Zero)}
    {Operation::Dyadic(Width::Width16, Subtract, X, random_stm8_operand(), Datum::Zero)}
    {Operation::Dyadic(Width::Width16, Subtract, Y, random_stm8_operand(), Datum::Zero)}
    )
}

fn transfers() -> Operation {
    randomly!(
    {Operation::Move(A, XL)}
    {Operation::Move(A, XH)}
    {Operation::Move(A, YL)}
    {Operation::Move(A, YH)}
    {Operation::Move(XL, A)}
    {Operation::Move(XH, A)}
    {Operation::Move(YL, A)}
    {Operation::Move(YH, A)}
    {Operation::Exchange(Width::Width8, A, XL)}
    {Operation::Exchange(Width::Width8, A, YL)}
    {Operation::Exchange(Width::Width16, X, Y)}
    {Operation::Move(random_imm16(), X)}
    {Operation::Move(random_imm16(), Y)}
    {Operation::Move(random_immediate(), random_absolute())}
    {Operation::Move(random_absolute(), random_absolute())}
    {Operation::Move(random_absolute(), A)}
    )
}

pub fn jumps() -> Operation {
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
        { Test::HalfCarry(random())}
        { Test::Overflow(random())}
        { Test::Carry(random())}
        { Test::SignedLowerThanOrEqual }
        { Test::SignedLowerThan }
        { Test::SignedGreaterThanOrEqual }
        { Test::SignedGreaterThan }
        { Test::UnsignedGreaterThan }
        { Test::UnsignedLowerThanOrEqual }
        { Test::Bit(random(), rand::thread_rng().gen_range(0..7), random())}
        )
    }

    Operation::Jump(cond(), j())
}

fn oneargs() -> Operation {
    fn op(w: Width, a: Datum) -> Operation {
        let vs = vec![
            Complement,
            Negate,
            Increment,
            Decrement,
            Swap,
            LeftShiftArithmetic,
            RightShiftArithmetic,
            RotateLeftThruCarry,
            RotateRightThruCarry,
        ];
        let o = vs.choose(&mut rand::thread_rng());
        Operation::Monadic(w, *o.unwrap(), a, a)
    }

    if random() {
        op(Width::Width8, A)
    } else {
        let a = if random() { X } else { Y };
        op(Width::Width16, a)
    }
}

const RANDS: [Instruction; 9] = [
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: clear,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: transfers,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: bits,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: carry,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: compare,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: jumps,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: oneargs,
    },
    Instruction {
        implementation: standard_implementation,
        disassemble: dasm,
        length: instr_length_stm8,
        operation: Operation::Nop,
        randomizer: twoargs,
    },
    Instruction {
        implementation: impl_muldiv,
        disassemble: dasm_muldiv,
        length: instr_length_muldiv,
        operation: Operation::Nop,
        randomizer: muldiv,
    },
];

pub fn instr_stm8() -> Instruction {
    let mut op = *RANDS.choose(&mut rand::thread_rng()).unwrap();
    op.randomize();
    op
}

pub fn instr_length_stm8(insn: &Instruction) -> usize {
    fn y_prefix_penalty(r: Datum) -> usize {
        if r == Y {
            return 1;
        }
        if r == YH {
            return 1;
        }
        if r == YL {
            return 1;
        }
        0
    }

    fn addr_length(r: u16) -> usize {
        if r < 256 {
            1
        } else {
            2
        }
    }

    match insn.operation {
        Operation::Dyadic(Width::Width8, _, _, Datum::Imm8(_), _) => 2,
        Operation::Dyadic(Width::Width8, _, _, Datum::Absolute(addr), _) => 1 + addr_length(addr),
        Operation::Dyadic(Width::Width16, _, _, Datum::Imm8(_), r) => 3 + y_prefix_penalty(r),
        Operation::Dyadic(Width::Width16, _, _, Datum::Absolute(addr), _) => 1 + addr_length(addr),
        Operation::Monadic(Width::Width16, _, _, r) => 1 + y_prefix_penalty(r),
        Operation::Monadic(Width::Width8, _, _, A) => 1,
        Operation::Exchange(_, Datum::Register(_), Datum::Register(_)) => 1,
        Operation::Exchange(Width::Width16, X, Y) => 1,
        Operation::Move(Datum::Zero, A) => 1,
        Operation::Move(Datum::Zero, X) => 1,
        Operation::Move(Datum::Zero, Y) => 2,
        Operation::Move(Datum::Absolute(addr), A) => 1 + addr_length(addr),
        Operation::Move(Datum::Zero, Datum::Absolute(addr)) => 1 + addr_length(addr),
        Operation::Move(Datum::Register(_), Datum::Register(r)) => {
            1 + y_prefix_penalty(Datum::Register(r))
        }
        Operation::Move(Datum::Imm16(_), r) => 1 + y_prefix_penalty(r),
        Operation::Move(Datum::Imm8(_), Datum::Absolute(_)) => 3,
        Operation::Move(Datum::Absolute(from), Datum::Absolute(to)) => {
            if from < 256 && to < 256 {
                3
            } else {
                5
            }
        }
        Operation::BitSet(_, _) => 4,
        Operation::BitClear(_, _) => 4,
        Operation::BitComplement(_, _) => 4,
        Operation::BitCopyCarry(_, _) => 4,
        Operation::Overflow(false) => 1,
        Operation::Carry(_) => 1,
        Operation::ComplementCarry => 1,
        Operation::BitCompare(Datum::Absolute(addr), A) => 1 + addr_length(addr),
        Operation::BitCompare(Datum::Imm8(_), A) => 2,
        Operation::Jump(Test::Bit(_, _, _), _) => 5,
        Operation::Jump(Test::HalfCarry(_), _) => 3,
        Operation::Jump(Test::True, FlowControl::Forward(_)) => 2,
        Operation::Jump(Test::True, FlowControl::Backward(_)) => 2,
        Operation::Jump(_, _) => 2,
        Operation::Shift(_, A) => 1,
        Operation::Shift(_, X) => 1,
        Operation::Shift(_, Y) => 2,
        Operation::Shift(_, Datum::Absolute(addr)) => 1 + addr_length(addr),
        _ => 0,
    }
}

fn stm8_reg_by_name(name: &str) -> Result<Datum, &'static str> {
    match name {
        "a" => Ok(A),
        "x" => Ok(X),
        "y" => Ok(Y),
        "xl" => Ok(XL),
        "yl" => Ok(YL),
        "xh" => Ok(XH),
        "yh" => Ok(YH),
        _ => reg_by_name(name),
    }
}

pub const STM8: Machine = Machine {
    name: "stm8",
    random_insn: instr_stm8,
    reg_by_name: stm8_reg_by_name,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BasicBlock;

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

            // Does this instruction have a length?
            assert!(i.len() > 0, "No instruction length for {}", i);
        }
        assert!(found_it, "Couldn't find instruction {}", opcode);
    }

    #[test]
    fn reg_names() {
        assert_eq!(stm8_reg_by_name("a").unwrap(), A);
        assert_eq!(stm8_reg_by_name("xl").unwrap(), XL);
        assert_eq!(stm8_reg_by_name("yl").unwrap(), YL);
        assert_eq!(stm8_reg_by_name("xh").unwrap(), XH);
        assert_eq!(stm8_reg_by_name("yh").unwrap(), YH);
        assert_eq!(stm8_reg_by_name("x").unwrap(), X);
        assert_eq!(stm8_reg_by_name("y").unwrap(), Y);
        assert_eq!(stm8_reg_by_name("m6").unwrap(), Datum::Absolute(6));
        assert!(stm8_reg_by_name("n").is_err());
        assert!(stm8_reg_by_name("m").is_err());
    }

    #[test]
    fn instruction_set_stm8() {
        // pop, popw, push, pushw, need to think about how to implement a stack
        find_it("adc", &RANDS[7]);
        find_it("add", &RANDS[7]);
        find_it("addw", &RANDS[7]);
        find_it("and", &RANDS[7]);
        find_it("bccm", &RANDS[2]);
        find_it("bcp", &RANDS[2]);
        find_it("bcpl", &RANDS[2]);
        // Not bothering with break; strop will not generate software interrupts
        find_it("bres", &RANDS[2]);
        find_it("bset", &RANDS[2]);
        find_it("btjf", &RANDS[5]);
        find_it("btjt", &RANDS[5]);
        // Not bothering with call callf callr; strop does not generate code that calls subroutines
        find_it("ccf", &RANDS[3]);
        find_it("clr", &RANDS[0]);
        find_it("clrw", &RANDS[0]);
        find_it("cp", &RANDS[4]);
        find_it("cpw", &RANDS[4]);
        find_it("cpl", &RANDS[6]);
        find_it("cplw", &RANDS[6]);
        find_it("dec", &RANDS[6]);
        find_it("decw", &RANDS[6]);
        find_it("div", &RANDS[8]);
        find_it("divw", &RANDS[8]);
        find_it("exg", &RANDS[1]);
        find_it("exgw", &RANDS[1]);
        // Not bothering with halt; strop will not stop the CPU.
        find_it("inc", &RANDS[6]);
        find_it("incw", &RANDS[6]);
        // Not bothering with int iret; strop will not handle interrupts
        // Not bothering with jpf; strop does not support STM8's having more than 64K RAM.
        find_it("jrc", &RANDS[5]);
        find_it("jreq", &RANDS[5]);
        // Not bothering with jrf; it's effectively a NOP
        find_it("jrh", &RANDS[5]);
        // Not bothering with jrih jril jrm; strop will not handle interrupts
        find_it("jrmi", &RANDS[5]);
        find_it("jrnc", &RANDS[5]);
        find_it("jrne", &RANDS[5]);
        find_it("jrnh", &RANDS[5]);
        // Not bothering with jrnm; strop will not handle interrupts
        find_it("jrnv", &RANDS[5]);
        find_it("jrpl", &RANDS[5]);
        find_it("jrsge", &RANDS[5]);
        find_it("jrsgt", &RANDS[5]);
        find_it("jrsle", &RANDS[5]);
        find_it("jrslt", &RANDS[5]);
        // Not bothering with jrt because it seems like an alias of jra
        // Not bothering with jruge because it seems like an alias of jrnc
        find_it("jrugt", &RANDS[5]);
        find_it("jrule", &RANDS[5]);
        // Not bothering with jrult because it seems like an alias of jrc
        find_it("jrv", &RANDS[5]);
        find_it("ld a, xh", &RANDS[1]);
        find_it("ld yl, a", &RANDS[1]);
        // Not bothering with ldf; strop does not support STM8's having more than 64K RAM.
        find_it("ldw", &RANDS[1]);
        find_it("mov", &RANDS[1]);
        find_it("mul", &RANDS[8]);
        find_it("neg", &RANDS[6]);
        find_it("negw", &RANDS[6]);
        find_it("or", &RANDS[7]);
        // Not bothering with ret retf; strop does not generate code that calls subroutines
        find_it("rcf", &RANDS[3]);
        // Not bothering with rim; strop will not handle interrupts
        find_it("rlc", &RANDS[6]);
        find_it("rlcw", &RANDS[6]);
        find_it("rlwa", &RANDS[6]);
        find_it("rrc", &RANDS[6]);
        find_it("rrcw", &RANDS[6]);
        find_it("rrwa", &RANDS[6]);
        find_it("rvf", &RANDS[3]);
        find_it("sbc", &RANDS[7]);
        find_it("scf", &RANDS[3]);
        // Not bothering with sim; strop will not handle interrupts
        find_it("sla", &RANDS[8]);
        find_it("slaw", &RANDS[8]);
        find_it("sra", &RANDS[8]);
        find_it("sraw", &RANDS[8]);
        find_it("srl", &RANDS[8]);
        find_it("srlw", &RANDS[8]);
        find_it("sub", &RANDS[7]);
        find_it("subw", &RANDS[7]);
        find_it("swap", &RANDS[6]);
        find_it("swapw", &RANDS[6]);
        find_it("tnz", &RANDS[7]);
        find_it("tnzw", &RANDS[7]);
        // Not bothering with trap wfe wfi; strop will not handle interrupts
        find_it("xor", &RANDS[7]);
    }
}
