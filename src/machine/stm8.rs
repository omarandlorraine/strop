use crate::machine::rand::prelude::SliceRandom;
use crate::machine::reg_by_name;
use rand::random;
use std::collections::HashMap;

pub struct IndexRegister {
    high: Option<u8>,
    low: Option<u8>,
}

pub struct State {
    a: Option<u8>,
    x: IndexRegister,
    y: IndexRegister,
    heap: HashMap<u16, Option<i8>>,
    carry: Option<bool>,
}

impl State {
    fn get_x(self) -> Option<u8> {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }

    fn get_y(self) -> Option<u8> {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }
}

enum BitSelect {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

impl From<usize> for BitSelect {
    fn from(item: usize) {
        match item {
            0 => BitSelect::B0,
            1 => BitSelect::B1,
            2 => BitSelect::B2,
            3 => BitSelect::B3,
            4 => BitSelect::B4,
            5 => BitSelect::B5,
            6 => BitSelect::B6,
            7 => BitSelect::B7,
        }
    }
}

impl From<BitSelect> for usize {
    fn from(item: usize) {
        match item {
            BitSelect::B0 => 0,
            BitSelect::B1 => 1,
            BitSelect::B2 => 2,
            BitSelect::B3 => 3,
            BitSelect::B4 => 4,
            BitSelect::B5 => 5,
            BitSelect::B6 => 6,
            BitSelect::B7 => 7,
        }
    }
}

// machine specific instruction operand
pub enum Operand {
    A,
    X,
    Y,
    Xh,
    Xl,
    Yh,
    Yl,
    Absolute(u16),
    Immediate8(u8),
    Immediate16(u16),
    IndX,
    IndY,
    // todo: more of these.
}

impl Operand {
    fn get8(self, s: &State) -> Option<u8> {
        match self {
            Operand::A => s.a,
            Operand::X => panic!(),
            Operand::Y => panic!(),
            Operand::Xh => s.x.high,
            Operand::Xl => s.x.low,
            Operand::Yh => s.y.high,
            Operand::Yl => s.y.low,
            Operand::Absolute(addr) => s.heap.get(&addr),
            Operand::Immediate8(x) => x,
            Operand::IndX => s.get_x().map(|addr| s.heap.get(addr)),
            Operand::IndY => s.get_y().map(|addr| s.heap.get(addr)),
        }
    }
}

impl rand::prelude::Distribution<Operand> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Operand {
        use self::Operand::*;
        rng.choose(&[
            A,
            X,
            Y,
            Xh,
            Xl,
            Yh,
            Yl,
            Absolute(random()),
            Immediate8(random()),
            IndX,
            IndY,
        ])
        .unwrap()
        .clone()
    }
}

pub enum Operands {
    None,
    One(Operand),
    Two(Operand, Operand),
    Bits(Operand, BitSelect),
}

impl Operands {
    fn one(self) -> Option<Operand> {
        match self {
            Self::One(o) => Some(o),
            _ => None,
        }
    }

    fn two(self) -> Option<(Operand, Operand)> {
        match self {
            Self::Two(a, b) => Some((a, b)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::Xl => write!(f, "xl"),
            Self::Yh => write!(f, "yh"),
            Self::Xl => write!(f, "xl"),
            Self::Yh => write!(f, "yh"),
            Self::Absolute(addr) => {
                if addr < &256 {
                    write!(f, "${:04x}", addr)
                } else {
                    write!(f, "${:06x}", addr)
                }
            }
            Self::Immediate8(byte) => write!(f, "#${:04x}", byte),
            Self::Immediate16(word) => write!(f, "#${:06x}", word),
            Self::IndX => write!(f, "(x)"),
            Self::IndY => write!(f, "(y)"),
        }
    }
}

pub type Operation<'a> =
    <crate::machine::Instruction<'a, crate::machine::stm8::State, Operands, (), ()>>::Operation;
pub type Instruction = crate::machine::Instruction<'static, State, Operands, (), ()>;

fn rotate_left_through_accumulator(s: &mut State, ind: Operand) {
    let ind = match ind {
        Operand::X => s.x,
        Operand::Y => s.y,
        _ => unimplemented!(),
    };

    let tmp = s.a;
    s.a = ind.high;
    ind.high = ind.low;
    ind.low = tmp;
}

fn rotate_right_through_accumulator(s: &mut State, ind: Operand) {
    let ind = match ind {
        Operand::X => s.x,
        Operand::Y => s.y,
        _ => unimplemented!(),
    };

    let tmp = s.a;
    s.a = ind.low;
    ind.low = ind.high;
    ind.high = tmp;
}

fn flip_x_and_y(d: Operand, sz: usize) -> (Operand, usize) {
    // Many STM8 instructions operate on either X or Y register. For some
    // instructions, this is controlled by an instruction prefix, which of
    // course needs to be accounted for in the instruction's length.
    // So this function changes X for Y and vice versa, and computes the new
    // length. This new length may of course be either discarded or written to
    // the Instruction's length field.
    match d {
        Operand::X => (Operand::Y, sz + 1),
        Operand::Y => (Operand::X, sz - 1),
        Operand::XL => (Operand::YL, sz + 1),
        Operand::YL => (Operand::XL, sz - 1),
        Operand::XH => (Operand::YH, sz + 1),
        Operand::YH => (Operand::XH, sz - 1),
        x => (x, sz),
    }
}

fn stm8_reg_by_name(name: &str) -> Result<Operand, &'static str> {
    match name {
        "a" => Ok(Operand::A),
        "x" => Ok(Operand::X),
        "y" => Ok(Operand::Y),
        "xl" => Ok(Operand::XL),
        "yl" => Ok(Operand::YL),
        "xh" => Ok(Operand::XH),
        "yh" => Ok(Operand::YH),
        _ => reg_by_name(name),
    }
}

pub const STM8: crate::Machine<State, Operand, (), ()> = crate::Machine {
    name: "stm8",
    random_insn: instr_stm8,
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
