use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Instruction;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::R;
use crate::machine::FlowControl;
use crate::machine::Test;
use crate::Datum;
use crate::Machine;

use crate::machine::rand::Rng;
use rand::random;

fn random_stm8_operand() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn random_register() -> Datum {
    if random() {
        Datum::Register(R::A)
    } else if random() {
        Datum::RegisterPair(R::Xh, R::Xl)
    } else {
        Datum::RegisterPair(R::Yh, R::Yl)
    }
}

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn bit(f: &mut std::fmt::Formatter, s: &'static str, d: u16, bitnumber: u8) -> std::fmt::Result {
        write!(f, "\t{}, ${:4}, #{}", s, d, bitnumber)

    }
    fn syn(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        match d {
            Datum::Imm8(val) => write!(f, "\t{} #${:2}", s, val),
            Datum::Absolute(addr) if addr < 256 => write!(f, "\t {} ${:2}", s, addr),
            Datum::Absolute(addr) => write!(f, "\t {} ${:4}", s, addr),
            Datum::Register(R::A) => write!(f, "\t {} a", s),
            Datum::RegisterPair(R::Xh, R::Xl) => write!(f, "\t {}w x", s),
            Datum::RegisterPair(R::Yh, R::Yl) => write!(f, "\t {}w y", s),
            _ => write!(f, "{} {:?}", s, d),
        }
    }

    fn dsyn(f: &mut std::fmt::Formatter, s: &'static str, r: Datum, d: Datum) -> std::fmt::Result {
        let (suffix, regname) = match r {
            Datum::Register(R::A) => ("", "a"),
            Datum::RegisterPair(R::Xh, R::Xl) => ("w", "x"),
            Datum::RegisterPair(R::Yh, R::Yl) => ("w", "y"),
            _ => panic!(),
        };

        match d {
            Datum::Imm8(val) => write!(f, "\t{}{} {}, #${:4}", s, suffix, regname, val),
            Datum::Absolute(addr) if addr < 256 => write!(f, "\t {}{} {}, ${:2}", s, suffix, regname, addr),
            Datum::Absolute(addr) => write!(f, "\t {}{} {}, ${:4}", s,suffix, regname,  addr),
            Datum::Register(R::A) => write!(f, "\t {}{} {}, a", suffix, regname, s),
            _ => write!(f, "{}{} {}, {:?}", s,suffix, regname,  d),
        }

    }

    fn regname(r: R) -> &'static str {
        match r {
            R::A => "a",
            R::Xh => "xh",
            R::Xl => "xl",
            R::Yh => "yh",
            R::Yl => "yl",
            _ => panic!()
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

    fn jump(f: &mut std::fmt::Formatter, s: &'static str, target: FlowControl) -> std::fmt::Result {
        match target {
            FlowControl::Forward(offs) => write!(f, "\t{} +{}", s, offs),
            FlowControl::Backward(offs) => write!(f, "\t{} -{}", s, offs),
            FlowControl::Invalid => panic!(),
            FlowControl::FallThrough => panic!(),
        }
    }

    match op {
        Operation::Add(d, r, true) => dsyn(f, "adc", r, d),
        Operation::Add(d, r, false) => dsyn(f, "add", r, d),
        Operation::And(d, r) => dsyn(f, "and", r, d),
        Operation::Compare(d, r) => dsyn(f, "cp", r, d),
        Operation::BitCompare(d, r) => dsyn(f, "bcp", r, d),
        Operation::Or(d, r) => dsyn(f, "or", r, d),
        Operation::Xor(d, r) => dsyn(f, "xor", r, d),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, d) => syn(f, "rlc", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => syn(f, "rrc", d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => syn(f, "sla", d),
        Operation::Shift(ShiftType::RightArithmetic, d) => syn(f, "sra", d),
        Operation::Move(Datum::Zero, r) => syn(f, "clr", r),
        Operation::Increment(r) => syn(f, "inc", r),
        Operation::Decrement(r) => syn(f, "dec", r),
        Operation::Move(Datum::Register(from), Datum::Register(to)) => write!(f, "\tld {}, {}", regname(to), regname(from)),
        Operation::BitClear(Datum::Absolute(addr), bitnumber) => bit(f, "bres", addr, bitnumber),
        Operation::BitSet(Datum::Absolute(addr), bitnumber) => bit(f, "bset", addr, bitnumber),
        Operation::BitCopyCarry(Datum::Absolute(addr), bitnumber) => bit(f, "bccm", addr, bitnumber),
        Operation::Carry(false) => write!(f, "\trcf"),
        Operation::Carry(true) => write!(f, "\tscf"),
        Operation::ComplementCarry => write!(f, "\tccf"),
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

fn add_adc(_mach: Machine) -> Operation {
    Operation::Add(random_stm8_operand(), random_register(), random())
}

fn bits(_mach: Machine) -> Operation {
    // the eight-bit diadic operations like and, xor, or, etc
    match rand::thread_rng().gen_range(0, 3) {
        0 => Operation::BitSet(random_absolute(), rand::thread_rng().gen_range(0, 7)),
        1 => Operation::BitClear(random_absolute(), rand::thread_rng().gen_range(0, 7)),
        _ => Operation::BitCopyCarry(random_absolute(), rand::thread_rng().gen_range(0, 7)),
    }
}

fn alu8(_mach: Machine) -> Operation {
    // the eight-bit diadic operations like and, xor, or, etc
    match rand::thread_rng().gen_range(0, 3) {
        0 => Operation::And(random_stm8_operand(), Datum::Register(R::A)),
        1 => Operation::Or(random_stm8_operand(), Datum::Register(R::A)),
        _ => Operation::Xor(random_stm8_operand(), Datum::Register(R::A)),
    }
}

fn shifts(_mach: Machine) -> Operation {
    // TODO: instructions SRA or SRAW.
    // TODO: instructions RLWA or RRWA.
    let sht = match rand::thread_rng().gen_range(0, 4) {
        0 => ShiftType::LeftArithmetic,
        1 => ShiftType::RightArithmetic,
        2 => ShiftType::RightRotateThroughCarry,
        _ => ShiftType::LeftRotateThroughCarry,
    };

    let operand = if random() {
        random_absolute()
    } else {
        random_register()
    };

    Operation::Shift(sht, operand)
}

fn carry(_mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 3) {
        0 => Operation::Carry(false),
        1 => Operation::Carry(true),
        _ => Operation::ComplementCarry
    }
}

fn compare(_mach: Machine) -> Operation {
    match rand::thread_rng().gen_range(0, 4) {
        0 => Operation::Compare(random_stm8_operand(), Datum::Register(R::A)),
        1 => Operation::Compare(random_stm8_operand(), Datum::RegisterPair(R::Xh, R::Xl)),
        2 => Operation::Compare(random_stm8_operand(), Datum::RegisterPair(R::Yh, R::Yl)),
        _ => Operation::BitCompare(random_stm8_operand(), Datum::Register(R::A)),
    }
}

fn incdec(_mach: Machine) -> Operation {
    let operand = if random() {
        random_absolute()
    } else {
        random_register()
    };
    
    if random() {
        Operation::Increment(operand)
    } else {
        Operation::Decrement(operand)
    }
}

fn transfers(_mach: Machine) -> Operation {
    fn rando() -> Datum {
        match rand::thread_rng().gen_range(0, 5) {
            0 => Datum::Register(R::A),
            1 => Datum::Register(R::Xl),
            2 => Datum::Register(R::Xh),
            3 => Datum::Register(R::Yl),
            _ => Datum::Register(R::Yh),
        }
    }
    match rand::thread_rng().gen_range(0, 3) {
        0 => Operation::Move(rando(), rando()), // Register-to-register
        1 => Operation::Move(Datum::Register(R::A), random_absolute()), // store accumulator
        _ => Operation::Move(random_stm8_operand(), Datum::Register(R::A)) // load accumulator
    }
}

pub fn jumps(_mach: Machine) -> Operation {
    fn j() -> FlowControl {
        // TODO: backward jumps.
        FlowControl::Forward(rand::thread_rng().gen_range(0, 3))
    }

    fn cond() -> Test {
        if random() {
            Test::True
        } else {
            Test::Carry(random())
        }
    }

    Operation::Jump(cond(), j())
}

pub fn instr_stm8(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 10) {
        0 => Instruction::new(mach, add_adc, dasm),
        1 => Instruction::new(mach, clear, dasm),
        2 => Instruction::new(mach, incdec, dasm),
        3 => Instruction::new(mach, transfers, dasm),
        4 => Instruction::new(mach, alu8, dasm),
        5 => Instruction::new(mach, bits, dasm),
        6 => Instruction::new(mach, carry, dasm),
        7 => Instruction::new(mach, compare, dasm),
        8 => Instruction::new(mach, jumps, dasm),
        _ => Instruction::new(mach, shifts, dasm),
    }
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
        find_it("adc", add_adc);
        find_it("add", add_adc);
        find_it("addw", add_adc);
        find_it("and", alu8);
        find_it("bccm", bits);
        find_it("bcp", compare);
        // TODO: bcpl btjf btjt
        find_it("bset", bits);
        find_it("bres", bits);
        // I don't think we need call, callf or callr
        // TODO: cpl cplw div divw exg exgw
        find_it("cp", compare);
        find_it("cpw", compare);
        find_it("ccf", carry);
        find_it("clr", clear);
        find_it("clrw", clear);
        find_it("dec", incdec);
        find_it("decw", incdec);
        // I don't think we need halt, iret
        find_it("inc", incdec);
        find_it("incw", incdec);
        // TODO: conditional jumps, relative jump
        find_it("jrc", jumps);
        // I don't think we need jrf, jrih, jril, jrm
        find_it("jrnc", jumps);
        find_it("ld a, xh", transfers);
        find_it("ld yl, a", transfers);
        // TODO: ld ldw mov mul neg negw
        // I don't think we need nop
        find_it("or", alu8);
        // TODO: pop popw push pushw
        find_it("rcf", carry);
        // I don't think we need ret, retf, rim
        // TODO: rlwa rrwa rvf sbc
        find_it("scf", carry);
        find_it("rlc", shifts);
        find_it("rlcw", shifts);
        find_it("rrc", shifts);
        find_it("rrcw", shifts);
        // I don't think we need sim
        // TODO: sra sraw srl srlw sub subw swap tnz tnzw
        find_it("sla", shifts); // aka. sll
        find_it("slaw", shifts); // aka. sllw
        // I don't think we need trap, wfe, wfi
        find_it("xor", alu8);
    }
}
