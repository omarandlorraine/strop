use crate::machine::arithmetic_shift_left;
use crate::machine::arithmetic_shift_right;
use crate::machine::logical_shift_right;
use crate::machine::rand::prelude::SliceRandom;
use crate::machine::random_shamt;
use crate::machine::randomize_shamt;
use crate::machine::reg_by_name;
use crate::machine::rotate_left;
use crate::machine::rotate_right;
use crate::machine::standard_add;
use crate::machine::standard_and;
use crate::machine::standard_bit_clear;
use crate::machine::standard_bit_complement;
use crate::machine::standard_bit_set;
use crate::machine::standard_complement;
use crate::machine::standard_decrement;
use crate::machine::standard_increment;
use crate::machine::standard_negate;
use crate::machine::standard_or;
use crate::machine::standard_subtract;
use crate::machine::standard_xor;
use std::collections::HashMap;
use strop::randomly;

pub struct IndexRegister {
    high: Some(u8),
    low: Some(u8),
}

pub struct State {
    a: Some(u8),
    x: IndexRegister,
    y: IndexRegister,
    heap: HashMap<u16, Option<i8>>,
    carry: Option<bool>,
}

impl State {
    fn get_x(self) -> Some(u16) {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }

    fn get_y(self) -> Some(u16) {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
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
    fn get8(s: &State) -> Option<u8> {
        match self {
            A => s.a,
            X => panic!(),
            Y => panic!(),
            Xh => s.x.high,
            Xl => s.x.low,
            Yh => s.y.high,
            Yl => s.y.low,
            Absolute(addr) => s.heap.get(&address),
            Immediate8(x) => x,
            IndX => s.get_x().map(|addr| s.heap.get(addr)),
            IndY => s.get_y().map(|addr| s.heap.get(addr)),
        }
    }
}

pub enum Operands {
    None,
    One(Operand),
    Two(Operand, Operand),
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
                if addr < 256 {
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
    <crate::machine::Instruction<'a, stm8::State, Operands, (), ()>>::Operation;
pub type Instruction = crate::machine::Instruction<'static, State, Operands, (), ()>;

fn adc(insn: &Instruction, s: &mut State) {
    let n = standard_add(s, s.get_i8(insn.a), s.get_i8(insn.b), s.carry);
    s.set_i8(insn.a, n);
}

fn add(insn: &Instruction, s: &mut State) {
    let n = standard_add(s, s.get_i8(insn.a), s.get_i8(insn.b), Some(false));
    s.set_i8(insn.a, n);
}

fn addw(insn: &Instruction, s: &mut State) {
    let n = standard_add(s, s.get_i16(insn.a), s.get_i16(insn.b), Some(false));
    s.set_i16(insn.a, n);
}

fn and(insn: &Instruction, s: &mut State) {
    let n = standard_and(s, s.get_i8(insn.a), s.get_i8(insn.b));
    s.set_i8(insn.a, n);
}

fn andw(insn: &Instruction, s: &mut State) {
    let n = standard_and(s, s.get_i16(insn.a), s.get_i16(insn.b));
    s.set_i16(insn.a, n);
}

fn bccm(insn: &Instruction, s: &mut State) {
    let datum = s.get_u8(insn.a);
    let shamt: Option<usize> = s.get_u8(insn.b).map(|v| v.into());
    if let Some(c) = s.carry {
        if c {
            s.set_u8(insn.a, standard_bit_clear(datum, shamt));
        } else {
            s.set_u8(insn.a, standard_bit_set(datum, shamt));
        }
    } else {
        s.set_u8(insn.a, None);
    }
}

fn bcpl(insn: &Instruction, s: &mut State) {
    let datum = s.get_u8(insn.a);
    let shamt: Option<usize> = s.get_u8(insn.b).map(|v| v.into());
    s.set_u8(insn.a, standard_bit_complement(datum, shamt));
}

fn bres(insn: &Instruction, s: &mut State) {
    let datum = s.get_u8(insn.a);
    let shamt: Option<usize> = s.get_u8(insn.b).map(|v| v.into());
    s.set_u8(insn.a, standard_bit_clear(datum, shamt));
}

fn bset(insn: &Instruction, s: &mut State) {
    let datum = s.get_u8(insn.a);
    let shamt: Option<usize> = s.get_u8(insn.b).map(|v| v.into());
    s.set_u8(insn.a, standard_bit_set(datum, shamt));
}

fn cp(insn: &Instruction, s: &mut State) {
    standard_subtract(s, s.get_u8(insn.a), s.get_u8(insn.b), Some(false));
}

fn cpw(insn: &Instruction, s: &mut State) {
    standard_subtract(s, s.get_u16(insn.a), s.get_u16(insn.b), Some(false));
}

fn cpl(insn: &Instruction, s: &mut State) {
    let d = standard_complement(s, s.get_u8(insn.a));
    s.set_u8(insn.a, d);
}

fn dec(insn: &Instruction, s: &mut State) {
    let d = standard_decrement(s, s.get_u8(insn.a));
    s.set_u8(insn.a, d);
}

fn inc(insn: &Instruction, s: &mut State) {
    let d = standard_increment(s, s.get_u8(insn.a));
    s.set_u8(insn.a, d);
}

fn ld(insn: &Instruction, s: &mut State) {
    s.set_u8(insn.a, s.get_u8(insn.b));
}

fn neg(insn: &Instruction, s: &mut State) {
    let d = standard_negate(s, s.get_i8(insn.a));
    s.set_i8(insn.a, d);
}

fn or(insn: &Instruction, s: &mut State) {
    let n = standard_or(s, s.get_i8(insn.a), s.get_i8(insn.b));
    s.set_i8(insn.a, n);
}

fn swap(insn: &Instruction, s: &mut State) {
    let d = s.get_i8(insn.a).map(|val| val.rotate_right(4));
    s.set_i8(insn.a, d);
}

fn swapw(insn: &Instruction, s: &mut State) {
    let d = s.get_i16(insn.a).map(|val| val.swap_bytes());
    s.set_i16(insn.a, d);
}

fn sla(insn: &Instruction, s: &mut State) {
    let n = arithmetic_shift_left(s, s.get_u8(insn.a));
    s.set_u8(insn.a, n);
}

fn sra(insn: &Instruction, s: &mut State) {
    let n = arithmetic_shift_right(s, s.get_u8(insn.a));
    s.set_u8(insn.a, n);
}

fn srl(insn: &Instruction, s: &mut State) {
    let n = logical_shift_right(s, s.get_u8(insn.a));
    s.set_u8(insn.a, n);
}

fn rlc(insn: &Instruction, s: &mut State) {
    let n = rotate_left(s, s.get_u8(insn.a));
    s.set_u8(insn.a, n);
}

fn rrc(insn: &Instruction, s: &mut State) {
    let n = rotate_right(s, s.get_u8(insn.a));
    s.set_u8(insn.a, n);
}

fn sbc(insn: &Instruction, s: &mut State) {
    let n = standard_subtract(s, s.get_i8(insn.a), s.get_i8(insn.b), s.carry);
    s.set_i8(insn.a, n);
}

fn sub(insn: &Instruction, s: &mut State) {
    let n = standard_subtract(s, s.get_i8(insn.a), s.get_i8(insn.b), Some(false));
    s.set_i8(insn.a, n);
}

fn subw(insn: &Instruction, s: &mut State) {
    let n = standard_subtract(s, s.get_i16(insn.a), s.get_i16(insn.b), Some(false));
    s.set_i16(insn.a, n);
}

fn xor(insn: &Instruction, s: &mut State) {
    let n = standard_xor(s, s.get_i8(insn.a), s.get_i8(insn.b));
    s.set_i8(insn.a, n);
}

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

fn flip_x_and_y(d: Datum, sz: usize) -> (Datum, usize) {
    // Many STM8 instructions operate on either X or Y register. For some
    // instructions, this is controlled by an instruction prefix, which of
    // course needs to be accounted for in the instruction's length.
    // So this function changes X for Y and vice versa, and computes the new
    // length. This new length may of course be either discarded or written to
    // the Instruction's length field.
    match d {
        X => (Y, sz + 1),
        Y => (X, sz - 1),
        XL => (YL, sz + 1),
        YL => (XL, sz - 1),
        XH => (YH, sz + 1),
        YH => (XH, sz - 1),
        x => (x, sz),
    }
}

const LOAD_INSTRUCTIONS: Instruction = Instruction {
    implementation: ld,
    disassemble: dasm_load,
    length: 2,
    randomizer: loads,
    a: A,
    b: Datum::Imm8(0),
    c: Datum::Nothing,
    mnemonic: "ld",
};

const ALU8_INSTRUCTIONS: Instruction = Instruction {
    implementation: add,
    disassemble: dasm_alu,
    length: 2,
    randomizer: alu8,
    a: A,
    b: Datum::Imm8(1),
    c: Datum::Nothing,
    mnemonic: "add",
};

const ALU16_INSTRUCTIONS: Instruction = Instruction {
    implementation: addw,
    disassemble: dasm_alu,
    length: 2,
    randomizer: alu16,
    a: X,
    b: Datum::Imm8(1),
    c: Datum::Nothing,
    mnemonic: "addw",
};

const MUL_DIV_INSTRUCTIONS: Instruction = Instruction {
    implementation: mul,
    disassemble: dasm_muldiv,
    length: 1,
    randomizer: muldiv,
    a: A,
    b: X,
    c: Datum::Nothing,
    mnemonic: "mul",
};

const CLEAR_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_one,
    length: 1,
    randomizer: clear,
    implementation: ld,
    a: A,
    b: Datum::Zero,
    c: Datum::Nothing,
    mnemonic: "clr",
};

const RMW_INSTRUCTIONS: Instruction = Instruction {
    implementation: sla,
    disassemble: dasm_one,
    length: 1,
    randomizer: oneargs,
    a: A,
    b: Datum::Zero,
    c: Datum::Nothing,
    mnemonic: "asl",
};

const FLAG_INSTRUCTIONS: Instruction = Instruction {
    implementation: |i, s| s.carry = Some(false),
    disassemble: dasm_inherent,
    length: 1,
    randomizer: carry,
    a: Datum::Nothing,
    b: Datum::Nothing,
    c: Datum::Nothing,
    mnemonic: "rcf",
};

const TRANSFER_INSTRUCTIONS: Instruction = Instruction {
    implementation: ld,
    disassemble: dasm_transfer,
    length: 1,
    randomizer: transfers,
    a: A,
    b: XH,
    c: Datum::Nothing,
    mnemonic: "ld",
};

const BIT_INSTRUCTIONS: Instruction = Instruction {
    implementation: bset,
    disassemble: dasm_bits,
    length: 4,
    randomizer: bits,
    a: Datum::Absolute(0),
    b: Datum::Imm8(0),
    c: Datum::Nothing,
    mnemonic: "bset",
};

const RANDS: [Instruction; 9] = [
    LOAD_INSTRUCTIONS,
    MUL_DIV_INSTRUCTIONS,
    ALU8_INSTRUCTIONS,
    ALU16_INSTRUCTIONS,
    CLEAR_INSTRUCTIONS,
    RMW_INSTRUCTIONS,
    FLAG_INSTRUCTIONS,
    TRANSFER_INSTRUCTIONS,
    BIT_INSTRUCTIONS,
];

pub fn instr_stm8() -> Instruction {
    let mut op = *RANDS.choose(&mut rand::thread_rng()).unwrap();
    op.randomize();
    op
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
