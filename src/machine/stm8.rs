use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::Instruction;
use crate::machine::Operation;
use crate::machine::ShiftType;
use crate::machine::R;
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
    } else {
        if random() {
            Datum::RegisterPair(R::Xh, R::Xl)
        } else {
            Datum::RegisterPair(R::Xh, R::Xl)
        }
    }
}

fn dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    match op {
        Operation::Add(d, r, true) => dsyn(f, "adc", r, d),
        Operation::Add(d, r, false) => dsyn(f, "add", r, d),
        Operation::Shift(ShiftType::LeftRotateThroughCarry, d) => syn(f, "rlc", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => syn(f, "rrc", d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => syn(f, "sla", d),
        Operation::Shift(ShiftType::RightArithmetic, d) => syn(f, "sra", d),
        Operation::Move(Datum::Zero, r) => syn(f, "clr", r),
        _ => write!(f, "{:?}", op),
    }
}

fn clear(_mach: Machine) -> Operation {
    if random() {
        Operation::Move(Datum::Zero, random_register())
    } else {
        Operation::Move(Datum::Zero, random_stm8_operand())
    }
}

fn add_adc(_mach: Machine) -> Operation {
    Operation::Add(random_stm8_operand(), random_register(), random())
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

pub fn instr_stm8(mach: Machine) -> Instruction {
    match rand::thread_rng().gen_range(0, 2) {
        0 => Instruction::new(mach, add_adc, dasm),
        1 => Instruction::new(mach, clear, dasm),
        _ => Instruction::new(mach, shifts, dasm),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn instruction_set_stm8() {
        find_it("adc", add_adc);
        find_it("add", add_adc);
        find_it("addw", add_adc);
        // TODO: and bccm bcp bcpl bres bset btjf btjt
        // I don't think we need call, callf or callr
        // TODO: ccf cp cpw cpl cplw dec decw div divw exg exgw
        find_it("clr", clear);
        find_it("clrw", clear);
        // I don't think we need halt
        // TODO: inc incw
        // I don't think we need iret
        // TODO: conditional jumps, relative jump
        // TODO: ld ldw mov mul neg negw
        // I don't think we need nop
        // TODO: or pop popw push pushw rcf
        // I don't think we need ret, retf, rim
        // TODO: rlwa rrwa rvf sbc scf
        find_it("rlc", shifts);
        find_it("rlcw", shifts);
        find_it("rrc", shifts);
        find_it("rrcw", shifts);
        // I don't think we need sim
        // TODO: sll sllw sra sraw srl srlw sub subw swap tnz tnzw
        find_it("sla", shifts);
        find_it("slaw", shifts);
        // I don't think we need trap, wfe, wfi
        // TODO: xor
        // note: IIRC some of the shift instructions are aliased to each other which the
        // disassembler is not going to care about, which means some of the TODOs up there are
        // automatically done. Just need to figure out which.
    }
}
