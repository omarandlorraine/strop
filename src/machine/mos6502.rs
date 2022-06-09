use crate::machine::rand::prelude::SliceRandom;
use crate::machine::rand::Rng;
use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::reg_by_name;
use crate::machine::standard_implementation;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{
    AddWithCarry, And, ExclusiveOr, Or, Subtract, SubtractWithCarry,
};
use crate::machine::FlowControl;
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::MonadicOperation::{
    Decrement, Increment, LeftShiftArithmetic, RightShiftLogical, RotateLeftThruCarry,
    RotateRightThruCarry,
};
use crate::machine::Operation;
use crate::machine::Test;
use crate::machine::Test::{Carry, Minus, Overflow, True, Zero};
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

    fn branch(f: &mut std::fmt::Formatter, s: &'static str, d: FlowControl) -> std::fmt::Result {
        match d {
            FlowControl::Backward(o) => {
                write!(f, "\t{} -{}", s, o)
            }
            FlowControl::Forward(o) => {
                write!(f, "\t{} +{}", s, o)
            }
            _ => {
                write!(f, "\t{} {:?}", s, d)
            }
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
        Operation::BitCompare(thing, A) => syn(f, "bit", thing),
        Operation::Dyadic(Width::Width8, Subtract, A, thing, Datum::Zero) => syn(f, "cmp", thing),
        Operation::Dyadic(Width::Width8, Subtract, X, thing, Datum::Zero) => syn(f, "cpx", thing),
        Operation::Dyadic(Width::Width8, Subtract, Y, thing, Datum::Zero) => syn(f, "cpy", thing),
        Operation::Dyadic(Width::Width8, SubtractWithCarry, A, thing, A) => syn(f, "sbc", thing),
        Operation::Dyadic(Width::Width8, AddWithCarry, A, thing, A) => syn(f, "adc", thing),
        Operation::Dyadic(Width::Width8, And, A, thing, A) => syn(f, "and", thing),
        Operation::Dyadic(Width::Width8, ExclusiveOr, A, thing, A) => syn(f, "eor", thing),
        Operation::Dyadic(Width::Width8, Or, A, thing, A) => syn(f, "ora", thing),
        Operation::Monadic(Width::Width8, LeftShiftArithmetic, d, _) => syn(f, "asl", d),
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
        Operation::Overflow(false) => write!(f, "\tclv"),
        Operation::Carry(false) => write!(f, "\tclc"),
        Operation::Carry(true) => write!(f, "\tsec"),
        Operation::Decimal(false) => write!(f, "\tcld"),
        Operation::Decimal(true) => write!(f, "\tsed"),
        Operation::Jump(Overflow(false), offs) => branch(f, "bvc", offs),
        Operation::Jump(Overflow(true), offs) => branch(f, "bvs", offs),
        Operation::Jump(Zero(false), offs) => branch(f, "bne", offs),
        Operation::Jump(Zero(true), offs) => branch(f, "beq", offs),
        Operation::Jump(Minus(false), offs) => branch(f, "bpl", offs),
        Operation::Jump(Minus(true), offs) => branch(f, "bmi", offs),
        Operation::Jump(Carry(false), offs) => branch(f, "bcc", offs),
        Operation::Jump(Carry(true), offs) => branch(f, "bcs", offs),
        Operation::Jump(True, offs) => branch(f, "jmp", offs),
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
        Operation::BitCompare(dat, A) => length(dat),
        Operation::Monadic(Width::Width8, _, dat, _) => length(dat),
        Operation::Dyadic(Width::Width8, _, _, dat, _) => length(dat),
        Operation::Overflow(_) => 1,
        Operation::Carry(_) => 1,
        Operation::Decimal(_) => 1,
        Operation::Jump(True, _) => 3,
        Operation::Jump(_, _) => 2,
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

fn rmw_dasm(op: Operation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fn syn(f: &mut std::fmt::Formatter, s: &'static str, d: Datum) -> std::fmt::Result {
        match d {
            Datum::Absolute(address) => {
                write!(f, "\t{} {}", s, address)
            }
            Datum::Register(R::A) => {
                write!(f, "\t{} a", s)
            }
            _ => {
                write!(f, "\t{} {:?}", s, d)
            }
        }
    }

    fn opcode(oper: MonadicOperation) -> &'static str {
        match oper {
            Decrement => "dec",
            Increment => "inc",
            LeftShiftArithmetic => "asl",
            RightShiftLogical => "lsr",
            RotateRightThruCarry => "ror",
            RotateLeftThruCarry => "rol",
            _ => panic!("I don't know the opcode for {:?}", oper),
        }
    }

    match op {
        Operation::Monadic(Width::Width8, Decrement, X, X) => {
            write!(f, "\tdex")
        }
        Operation::Monadic(Width::Width8, Decrement, Y, Y) => {
            write!(f, "\tdey")
        }
        Operation::Monadic(Width::Width8, Increment, X, X) => {
            write!(f, "\tinx")
        }
        Operation::Monadic(Width::Width8, Increment, Y, Y) => {
            write!(f, "\tiny")
        }
        Operation::Monadic(Width::Width8, oper, d, _) => syn(f, opcode(oper), d),
        _ => {
            write!(f, "{:?}", op)
        }
    }
}

fn rmw_op(cmos: bool) -> Operation {
    // Operations which can be performed on either memory or the accumulator
    let ma = vec![
        LeftShiftArithmetic,
        RightShiftLogical,
        RotateLeftThruCarry,
        RotateRightThruCarry,
    ];

    // Operations which can be performed on either memory or an index register
    let mxy = vec![Decrement, Increment];

    fn xy(cmos: bool) -> Datum {
        // Pick an index register (if CMOS this includes the accumulator)
        if cmos {
            *[X, Y, A].choose(&mut rand::thread_rng()).unwrap()
        } else {
            *[X, Y].choose(&mut rand::thread_rng()).unwrap()
        }
    }

    if random() {
        let op = *ma.choose(&mut rand::thread_rng()).unwrap();
        let d = if random() { A } else { random_absolute() };
        Operation::Monadic(Width::Width8, op, d, d)
    } else {
        let op = *mxy.choose(&mut rand::thread_rng()).unwrap();
        let d = if random() {
            xy(cmos)
        } else {
            random_absolute()
        };
        Operation::Monadic(Width::Width8, op, d, d)
    }
}

fn alu_6502() -> Operation {
    // randomly generate the instructions ora, and, eor, adc, sbc, cmp
    // these all have the same available addressing modes
    let ops = vec![AddWithCarry, And, Or, ExclusiveOr, SubtractWithCarry];
    let op = *ops.choose(&mut rand::thread_rng()).unwrap();
    Operation::Dyadic(Width::Width8, op, A, random_source(), A)
}

fn transfers_6502() -> Operation {
    let reg = if random() { X } else { Y };
    if random() {
        Operation::Move(A, reg)
    } else {
        Operation::Move(reg, A)
    }
}

fn loads() -> Operation {
    let reg = randomly!( { A } { X } { Y } );

    if random() {
        Operation::Move(random_absolute(), reg)
    } else {
        Operation::Move(random_immediate(), reg)
    }
}

fn stores() -> Operation {
    let reg = randomly!( { A } { X } { Y } );

    Operation::Move(reg, random_absolute())
}

fn secl_6502() -> Operation {
    randomly!(
        {Operation::Carry(random())}
        {Operation::Decimal(random())}
        {Operation::Overflow(false)}
    )
}

fn branches() -> Operation {
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
           { Test::Overflow(random())}
           { Test::Carry(random())})
    }

    Operation::Jump(cond(), j())
}

const BRANCH_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: branches,
};

fn compares() -> Operation {
    randomly!(
    { Operation::BitCompare(random_absolute(), A)}
    { Operation::Dyadic(Width::Width8, Subtract, A, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, X, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, Y, random_source(), Datum::Zero)}
    )
}

const COMPARE_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: compares,
};

const STORE_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: stores,
};

const LOAD_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: loads,
};

const TRANSFER_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: transfers_6502,
};

const ALU_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: alu_6502,
};

const RMW_NMOS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: rmw_dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: || rmw_op(false),
};

const RMW_CMOS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: rmw_dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: || rmw_op(true),
};

const FLAG_INSTRUCTIONS: Instruction = Instruction {
    implementation: standard_implementation,
    disassemble: dasm,
    length: instr_length_6502,
    operation: Operation::Nop,
    randomizer: secl_6502,
};

const NMOS6502_INSTRUCTIONS: [Instruction; 7] = [
    ALU_INSTRUCTIONS,
    FLAG_INSTRUCTIONS,
    RMW_NMOS,
    TRANSFER_INSTRUCTIONS,
    LOAD_INSTRUCTIONS,
    COMPARE_INSTRUCTIONS,
    BRANCH_INSTRUCTIONS,
];

const CMOS6502_INSTRUCTIONS: [Instruction; 7] = [
    ALU_INSTRUCTIONS,
    FLAG_INSTRUCTIONS,
    RMW_CMOS,
    TRANSFER_INSTRUCTIONS,
    LOAD_INSTRUCTIONS,
    COMPARE_INSTRUCTIONS,
    BRANCH_INSTRUCTIONS,
];

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
        for i in [
            "asl", "dec", "dex", "dey", "inc", "inx", "iny", "lsr", "rol", "ror",
        ] {
            find_it(i, &RMW_CMOS);
            find_it(i, &RMW_NMOS);
        }

        for i in ["tax", "txa", "tay", "tya"] {
            find_it(i, &TRANSFER_INSTRUCTIONS);
        }

        for i in ["clc", "sec", "clv", "sed", "cld"] {
            find_it(i, &FLAG_INSTRUCTIONS);
        }

        for i in ["adc", "and", "eor", "ora", "sbc"] {
            find_it(i, &ALU_INSTRUCTIONS);
        }

        for i in ["lda", "ldx", "ldy"] {
            find_it(i, &LOAD_INSTRUCTIONS);
        }

        for i in ["bit", "cmp", "cpx", "cpy"] {
            find_it(i, &COMPARE_INSTRUCTIONS);
        }

        for i in [
            "bcc", "bcs", "beq", "bmi", "bne", "bpl", "bvc", "bvs", "jmp",
        ] {
            find_it(i, &BRANCH_INSTRUCTIONS);
        }

        for i in ["sta", "stx", "sty"] {
            find_it(i, &STORE_INSTRUCTIONS);
        }

        // not bothering with nop; there's NO Point
        // not bothering with brk; it's some kind of buggy software interrupt instruction.
        // not bothering with cli; strop does not handle interrupts
        // not bothering with jsr; strop does not call subroutines
        // not bothering with rti; strop does not handle interrupts
        // not bothering with rts; strop does not call subroutines
        // not bothering with sei; strop does not handle interrupts
        // as for txs tsx pha pla php plp, we need ot figure out how/if we're going to implement a stack.
        // need to add stz for 65c02
        // need to add lax sax for 65n02
    }

    use crate::machine::mos6502::{A, X, Y};
    use crate::machine::DyadicOperation;
    use crate::machine::Operation;
    use crate::Datum;

    extern crate mos6502;
    use mos6502::address::Address;
    use mos6502::cpu;
    use mos6502::registers::Status;
    use rand::random;

    fn run_mos6502(
        opcode: u8,
        val1: u8,
        val2: u8,
        carry: bool,
        decimal: bool,
    ) -> (i8, bool, bool, bool, bool) {
        let mut cpu = cpu::CPU::new();

        let program = [
            // set or clear the carry flag
            if carry { 0x38 } else { 0x18 },
            // set or clear the decimal flag
            if decimal { 0xf8 } else { 0xd8 },
            // load val1 into the accumulator
            0xa9,
            val1,
            // run the opcode on val2
            opcode,
            val2,
            // stop the emulated CPU
            0xff,
        ];

        cpu.memory.set_bytes(Address(0x10), &program);
        cpu.registers.program_counter = Address(0x10);
        cpu.run();

        (
            cpu.registers.accumulator,
            cpu.registers.status.contains(Status::PS_ZERO),
            cpu.registers.status.contains(Status::PS_CARRY),
            cpu.registers.status.contains(Status::PS_NEGATIVE),
            cpu.registers.status.contains(Status::PS_OVERFLOW),
        )
    }

    fn run_strop(
        op: DyadicOperation,
        val1: u8,
        val2: u8,
        carry: bool,
        decimal: bool,
    ) -> (i8, bool, bool, bool, bool) {
        use crate::State;
        let mut c = FLAG_INSTRUCTIONS.clone();
        c.operation = Operation::Carry(carry);

        let mut d = FLAG_INSTRUCTIONS.clone();
        d.operation = Operation::Decimal(decimal);

        let mut lda1 = LOAD_INSTRUCTIONS.clone();
        lda1.operation = Operation::Move(Datum::Imm8(i8::from_ne_bytes(val1.to_ne_bytes())), A);

        let mut ope = ALU_INSTRUCTIONS.clone();
        ope.operation = Operation::Dyadic(
            Width::Width8,
            op,
            A,
            Datum::Imm8(i8::from_ne_bytes(val2.to_ne_bytes())),
            A,
        );

        let mut s = State::new();
        c.operate(&mut s);
        d.operate(&mut s);
        lda1.operate(&mut s);
        ope.operate(&mut s);

        (
            s.get_i8(A).unwrap(),
            s.zero.unwrap_or(false),
            s.carry.unwrap_or(false),
            s.sign.unwrap_or(false),
            s.overflow.unwrap_or(false),
        )
    }

    #[test]
    fn adc_set_carry() {
        // check that the carry flag is set when unsigned addition carries over
        assert!(
            run_strop(AddWithCarry, 0x94, 0x83, false, false).2,
            "adc instruction didn't set carry flag but should have"
        );
    }

    #[test]
    fn adc_set_sign() {
        // check that the sign flag is set when unsigned addition carries over
        assert!(
            run_strop(AddWithCarry, 0x31, 0xb5, false, false).3,
            "adc instruction didn't set sign flag but should have"
        );
    }

    #[test]
    fn adc_set_overflow() {
        // check that the overflow flag is set when unsigned addition carries over
        assert!(
            run_strop(AddWithCarry, 0x82, 0x87, true, false).4,
            "adc instruction didn't set overflow flag but should have"
        );
    }

    #[test]
    fn and_flags() {
        assert!(
            run_strop(And, 0xe8, 0x80, true, false).3,
            "and instruction didn't set sign flag but should've"
        );
        assert!(
            run_strop(And, 0x8e, 0x70, true, false).1,
            "and instruction didn't set zero flag but should've"
        );
    }

    fn fuzz_dyadic(op: DyadicOperation, opcode: u8) {
        for _i in 0..5000 {
            let a: u8 = random();
            let b: u8 = random();
            let c: bool = random();
            let d: bool = false;

            let msg = format!("For {:#04x} {:?}", opcode, op);
            let regr = format!("run_strop({:?}, {:#04x}, {:#04x}, {}, {})", op, a, b, c, d);
            let truth = run_mos6502(opcode, a, b, c, d);
            let strop = run_strop(op, a, b, c, d);

            assert!(
                truth.0 == strop.0,
                "{}, run {} and check accumulator == {}",
                msg,
                regr,
                truth.0
            );

            assert!(
                truth.1 == strop.1,
                "{}, run {} and check zero flag == {}",
                msg,
                regr,
                truth.1
            );

            if b != 0xff {
                // There should be no if here.
                // This is a workaround for a bug in the mos6502 crate
                assert!(
                    truth.2 == strop.2,
                    "{}, run {} and check carry == {}",
                    msg,
                    regr,
                    truth.2
                );
            }

            assert!(
                truth.3 == strop.3,
                "{}, run {} and check sign flag == {}",
                msg,
                regr,
                truth.3
            );

            assert!(
                truth.4 == strop.4,
                "{}, run {} and check overflow flag == {}",
                msg,
                regr,
                truth.4
            );
        }
    }

    #[test]
    fn fuzzer_call() {
        fuzz_dyadic(AddWithCarry, 0x69);
        fuzz_dyadic(And, 0x29);
        fuzz_dyadic(SubtractWithCarry, 0xe9);
    }
}
