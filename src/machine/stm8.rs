use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::FlowControl;
use crate::machine::R;
use crate::Datum;
use crate::Machine;
use crate::State;

use crate::machine::rand::prelude::SliceRandom;
use crate::machine::reg_by_name;
use crate::machine::Instruction;
use rand::random;

struct Opcode {
    handler: fn(&Stm8Instruction, &mut State) -> FlowControl,
    name: &'static str,
}

#[derive(Clone, Copy)]
pub struct Stm8Instruction {
    randomizer: fn(&mut Self),
    disassemble: fn(&Self, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    implementation: fn(&Self, &mut State) -> FlowControl,
}

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl Instruction for Stm8Instruction {
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
const XL: Datum = Datum::Register(R::Xl);
const YL: Datum = Datum::Register(R::Yl);
const XH: Datum = Datum::Register(R::Xh);
const YH: Datum = Datum::Register(R::Yh);
const X: Datum = Datum::RegisterPair(R::Xh, R::Xl);
const Y: Datum = Datum::RegisterPair(R::Yh, R::Yl);

fn random_imm16() -> Datum {
    let regs = vec![150];
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
    reg_by_name: stm8_reg_by_name,
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

            // Does this instruction have a length?
            assert!(i.len() > 0, "No instruction length for {}", i);
        }
        assert!(found_it, "Couldn't find instruction {}", opcode);
    }

    #[test]
    fn x_width() {
        let mut s = State::new();

        s.set_i16(X, Some(10));
        assert_eq!(10, s.get_i16(XL).unwrap());
        assert_eq!(0, s.get_i16(XH).unwrap());
        assert_eq!(10, s.get_u16(XL).unwrap());
        assert_eq!(0, s.get_u16(XH).unwrap());

        s.set_u16(X, Some(10));
        assert_eq!(10, s.get_u16(XL).unwrap());
        assert_eq!(0, s.get_u16(XH).unwrap());

        s.set_i16(X, Some(-1));
        assert_eq!(-1, s.get_i16(XL).unwrap());
        assert_eq!(-1, s.get_i16(XH).unwrap());

        s.set_i16(X, Some(150));
        assert_eq!(150, s.get_i16(X).unwrap());

        s.set_u16(X, Some(15000));
        assert_eq!(15000, s.get_u16(X).unwrap());

        s.set_i16(X, Some(0));
        assert_eq!(0, s.get_i16(XL).unwrap());
        assert_eq!(0, s.get_i16(XH).unwrap());
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
        find_it("ldw", &RANDS[9]);
        find_it("mov", &RANDS[9]);
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
        find_it("sla", &RANDS[6]);
        find_it("slaw", &RANDS[6]);
        find_it("sra", &RANDS[6]);
        find_it("sraw", &RANDS[6]);
        find_it("srl", &RANDS[6]);
        find_it("srlw", &RANDS[6]);
        find_it("sub", &RANDS[7]);
        find_it("subw", &RANDS[7]);
        find_it("swap", &RANDS[6]);
        find_it("swapw", &RANDS[6]);
        find_it("tnz", &RANDS[4]);
        find_it("tnzw", &RANDS[4]);
        // Not bothering with trap wfe wfi; strop will not handle interrupts
        find_it("xor", &RANDS[7]);
    }
}
