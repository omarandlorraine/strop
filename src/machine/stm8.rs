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
        let t = match d {
            Datum::Imm8(val) => format!("#${:2}", val),
            Datum::Absolute(addr) if addr < 256 => format!("${:2}", addr),
            Datum::Absolute(addr) => format!("${:4}", addr),
            Datum::Register(R::A) => "a".into(),
            _ => format!("{:?}", d),
        };
        write!(f, "\t{} {}", s, t)
    }

    match op {
        Operation::Add(d, Datum::Register(R::A), true) => syn(f, "adc", d),
        Operation::Add(d, Datum::Register(R::A), false) => syn(f, "add", d),
        Operation::Shift(ShiftType::RightRotateThroughCarry, d) => syn(f, "rrc", d),
        Operation::Shift(ShiftType::LeftArithmetic, d) => syn(f, "sla", d),
        _ => write!(f, "{:?}", op),
    }
}

fn add_adc(_mach: Machine) -> Operation {
    Operation::Add(random_stm8_operand(), Datum::Register(R::A), random())
}

fn shifts(_mach: Machine) -> Operation {
    // TODO: instructions SRA or SRAW.
    // TODO: instructions RLWA or RRWA.
    let sht = match rand::thread_rng().gen_range(0, 2) {
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
        // TODO: addw and bccm bcp bcpl bres bset btjf btjt
        // I don't think we need call, callf or callr
        // TODO: ccf clr clrw cp cpw cpl cplw dec decw div divw exg exgw
        // I don't think we need halt
        // TODO: inc inw
        // I don't think we need iret
        // TODO: conditional jumps, relative jump
        // TODO: ld ldw mov mul neg negw
        // I don't think we need nop
        // TODO: or pop popw push pushw rcf
        // I don't think we need ret, retf, rim
        // TODO: rlc rlcw rlwa rrc rrcw rrwa rvf sbc scf
        find_it("rrc", shifts);
        // I don't think we need sim
        // TODO: sla slaw sll sllw sra sraw srl srlw sub subw swap tnz tnzw
        find_it("sla", shifts);
        // I don't think we need trap, wfe, wfi
        // TODO: xor
        // note: IIRC some of the shift instructions are aliased to each other which the
        // disassembler is not going to care about, which means some of the TODOs up there are
        // automatically done. Just need to figure out which.
    }
}
