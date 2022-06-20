use crate::machine::rand::prelude::SliceRandom;
use crate::machine::reg_by_name;
use crate::machine::Datum;
use crate::machine::DyadicOperation::{
    AddWithCarry, And, ExclusiveOr, Or, Subtract, SubtractWithCarry,
};
use crate::machine::Instruction;
use crate::machine::Machine;
use crate::machine::MonadicOperation;
use crate::machine::MonadicOperation::{
    Decrement, Increment, LeftShiftArithmetic, RightShiftLogical, RotateLeftThruCarry,
    RotateRightThruCarry,
};
use crate::machine::Operation;
use crate::machine::Test::True;
use crate::machine::Width;
use crate::machine::R;

use crate::machine::standard_compare;
use rand::random;
use strop::randomly;

const A: Datum = Datum::Register(R::A);
const X: Datum = Datum::Register(R::Xl);
const Y: Datum = Datum::Register(R::Yl);

fn random_immediate() -> (Datum, usize) {
    let d = crate::machine::random_immediate();
    (d, 1)
}

fn random_absolute() -> (Datum, usize) {
    let d = crate::machine::random_absolute();
    match d {
        Datum::Absolute(addr) => (d, if addr < 256 { 2 } else { 3 }),
        _ => panic!(),
    }
}

fn dasm_no_operand(f: &mut std::fmt::Formatter<'_>, insn: &Instruction) -> std::fmt::Result {
    write!(f, "\t{}", insn.mnemonic)
}

fn dasm_operand(
    f: &mut std::fmt::Formatter<'_>,
    opcode: &'static str,
    operand: &Datum,
) -> std::fmt::Result {
    // This one prints out the opcode and one operand
    // examples:
    //     cpx #4
    //     lda 5
    //     stx 67
    //     ror a

    match operand {
        Datum::Absolute(address) => {
            write!(f, "\t{} {}", opcode, address)
        }
        Datum::Register(R::A) => {
            write!(f, "\t{} a", opcode)
        }
        Datum::Imm8(val) => {
            write!(f, "\t{} #{}", opcode, val)
        }
        _ => {
            write!(f, "\t{} {:?}", opcode, operand)
        }
    }
}

fn dasm_b(f: &mut std::fmt::Formatter<'_>, insn: &Instruction) -> std::fmt::Result {
    dasm_operand(f, insn.mnemonic, &insn.b)
}

fn rmw_dasm(f: &mut std::fmt::Formatter<'_>, insn: &Instruction) -> std::fmt::Result {
    // special cases for opcodes: dex dey inx iny
    if insn.mnemonic == "inc" && insn.a == X {
        write!(f, "\tinx")
    } else if insn.mnemonic == "dec" && insn.a == X {
        write!(f, "\tdex")
    } else if insn.mnemonic == "inc" && insn.a == Y {
        write!(f, "\tiny")
    } else if insn.mnemonic == "dec" && insn.a == Y {
        write!(f, "\tdey")
    } else {
        dasm_operand(f, insn.mnemonic, &insn.a)
    }
}

fn rmw_op(insn: &mut Instruction, cmos: bool) {
    fn is_inc_dec(insn: &Instruction) -> bool {
        // Is this instruction current either inc or dec?
        // This affects which operand may be in use.
        insn.implementation == increment || insn.implementation == decrement
    }

    fn is_x_y(insn: &Instruction) -> bool {
        insn.a == X || insn.a == Y
    }

    randomly!(
        { if cmos || !is_inc_dec(insn)
            // not all 6502's have instructions to increment or decrement the accumulator
            { insn.a = A; insn.length = 1; }
        }
        { if is_inc_dec(insn) { insn.a = X; insn.length = 1; } }
        { if is_inc_dec(insn) { insn.a = Y; insn.length = 1; } }
        { insn.a, insn.length = random_absolute(); }
        { insn.mnemonic = "inc"; insn.implementation = inc; }
        { insn.mnemonic = "dec"; insn.implementation = dec; }
        { if !is_x_y(insn) { insn.mnemonic = "asl"; insn.implementation = asl; } }
        { if !is_x_y(insn) { insn.mnemonic = "lsr"; insn.implementation = lsr; } }
        { if !is_x_y(insn) { insn.mnemonic = "ror"; insn.implementation = ror; } }
        { if !is_x_y(insn) { insn.mnemonic = "rol"; insn.implementation = rol; } }
    );
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

fn loads(insn: &Instruction) {
    randomly!(
        { insn.a = A; insn.mnemonic = "lda" }
        { insn.a = X; insn.mnemonic = "ldx" }
        { insn.a = Y; insn.mnemonic = "ldy" }
        { insn.b, insn.length = random_absolute(); }
        { insn.b, insn.length = random_immediate(); }
    );
}

fn stores(insn: &Instruction) {
    randomly!(
        { insn.a = A; insn.mnemonic = "sta" }
        { insn.a = X; insn.mnemonic = "stx" }
        { insn.a = Y; insn.mnemonic = "sty" }
        { insn.b, insn.length = random_absolute(); }
    );
}

fn secl_6502(insn: &mut Instruction) {
    (insn.mnemonic, insn.implementation) = randomly!(
        { ("clv", |_, s| s.overflow = Some(false)) }
        { ("clc", |_, s| s.carry = Some(false)) }
        { ("sec", |_, s| s.carry = Some(true)) }
        { ("cld", |_, s| s.decimal = Some(false)) }
        { ("sed", |_, s: &mut State| s.decimal = Some(true)) });
}

fn compares() -> Operation {
    randomly!(
    { Operation::BitCompare(random_absolute(), A)}
    { Operation::Dyadic(Width::Width8, Subtract, A, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, X, random_source(), Datum::Zero)}
    { Operation::Dyadic(Width::Width8, Subtract, Y, random_source(), Datum::Zero)}
    )
}

const COMPARE_INSTRUCTIONS: Instruction = Instruction {
    mnemonic: "cmp",
    a: A,
    b: Datum::Imm8(0),
    disassemble: dasm_operand,
    randomizer: compares,
};

const STORE_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_operand,
    randomizer: stores,
};

const LOAD_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_operand,
    randomizer: loads,
};

const TRANSFER_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_operand,
    randomizer: transfers_6502,
};

const ALU_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_operand,
    randomizer: alu_6502,
};

const RMW_NMOS: Instruction = Instruction {
    disassemble: rmw_dasm,
    randomizer: |insn| rmw_op(insn, false),
    implementation: asl,
    length: 1,
    mnemonic: "asl",
    a: A,
    b: Datum::Nothing,
    c: Datum::Nothing,
};

const RMW_CMOS: Instruction = Instruction {
    disassemble: rmw_dasm,
    randomizer: |insn| rmw_op(insn, true),
    implementation: asl,
    length: 1,
    mnemonic: "asl",
    a: A,
    b: Datum::Nothing,
    c: Datum::Nothing,
};

const FLAG_INSTRUCTIONS: Instruction = Instruction {
    disassemble: dasm_no_operand,
    implementation: |_, s| s.carry = Some(false),
    mnemonic: "clc",
    randomizer: secl_6502,
    length: 1,
    a: Datum::Nothing,
    b: Datum::Nothing,
    c: Datum::Nothing,
};

const NMOS6502_INSTRUCTIONS: [Instruction; 6] = [
    ALU_INSTRUCTIONS,
    FLAG_INSTRUCTIONS,
    RMW_NMOS,
    TRANSFER_INSTRUCTIONS,
    LOAD_INSTRUCTIONS,
    COMPARE_INSTRUCTIONS,
];

const CMOS6502_INSTRUCTIONS: [Instruction; 6] = [
    ALU_INSTRUCTIONS,
    FLAG_INSTRUCTIONS,
    RMW_CMOS,
    TRANSFER_INSTRUCTIONS,
    LOAD_INSTRUCTIONS,
    COMPARE_INSTRUCTIONS,
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
pub mod tests {
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

    fn run_strop_monadic(
        op: MonadicOperation,
        val1: u8,
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

        let mut ope = RMW_NMOS.clone();
        ope.operation = Operation::Monadic(Width::Width8, op, A, A);

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

    fn run_strop(
        op: &Instruction,
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

        let mut s = State::new();
        c.operate(&mut s);
        d.operate(&mut s);
        lda1.operate(&mut s);
        op.operate(&mut s);

        (
            s.get_i8(A).unwrap(),
            s.zero.unwrap_or(false),
            s.carry.unwrap_or(false),
            s.sign.unwrap_or(false),
            s.overflow.unwrap_or(false),
        )
    }

    #[test]
    fn adc_flags() {
        // check that the carry flag is set when unsigned addition carries over
        assert!(
            run_strop(AddWithCarry, 0x94, 0x83, false, false).2,
            "adc instruction didn't set carry flag but should have"
        );
        assert!(
            run_strop(AddWithCarry, 0x31, 0xb5, false, false).3,
            "adc instruction didn't set sign flag but should have"
        );
        assert!(
            run_strop(AddWithCarry, 0x82, 0x87, true, false).4,
            "adc instruction didn't set overflow flag but should have"
        );
    }

    #[test]
    fn asl_flags() {
        assert!(
            run_strop_monadic(LeftShiftArithmetic, 0x86, false, false).2,
            "asl instruction should've set carry flag but didn't"
        );
        assert!(
            run_strop_monadic(LeftShiftArithmetic, 0xde, false, false).3,
            "asl instruction didn't set sign flag but should have"
        );
        assert!(
            run_strop_monadic(LeftShiftArithmetic, 0x80, false, false).1,
            "asl instruction didn't set zero flag but should have"
        );
    }

    #[test]
    fn ora_flags() {
        assert!(
            run_strop(Or, 0x00, 0x00, true, false).1,
            "eor instruction didn't set zero flag but should've"
        );
        assert!(
            run_strop(Or, 0x04, 0xbe, true, false).3,
            "ora instruction didn't set sign flag but should've"
        );
    }

    #[test]
    fn eor_flags() {
        assert!(
            run_strop(ExclusiveOr, 0x37, 0x37, true, false).1,
            "eor instruction didn't set zero flag but should've"
        );
        assert!(
            run_strop(ExclusiveOr, 0x29, 0x87, false, false).3,
            "eor instruction didn't set sign flag but should've"
        );
    }

    #[test]
    fn ror_flags() {
        assert!(
            run_strop_monadic(RotateRightThruCarry, 0x5f, true, false).3,
            "ror instruction shouldn've set the sign flag but didn't"
        );
        assert!(
            run_strop_monadic(RotateRightThruCarry, 0x71, false, false).2,
            "ror instruction shouldn've set the carry flag but didn't"
        );
    }

    #[test]
    fn ror_result() {
        let result = run_strop_monadic(RotateRightThruCarry, 0x43, false, false).0;
        assert!(
            result == 0x21,
            "0x43 rotated right should be 0x21 but was found to be {:#04x}",
            result
        );
    }

    #[test]
    fn rol_flags() {
        assert!(
            !run_strop_monadic(RotateLeftThruCarry, 0x58, true, false).2,
            "rol instruction set carry flag but shouldn't've"
        );
    }

    #[test]
    fn lsr_flags() {
        assert!(
            !run_strop_monadic(RightShiftLogical, 0xab, false, false).3,
            "lsr instruction didn't set sign flag but should've"
        );
        assert!(
            !run_strop_monadic(RightShiftLogical, 0xde, false, false).2,
            "lsr instruction should've set carry flag but didn't"
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

    fn fuzz_monadic(op: MonadicOperation, opcode: u8) {
        for _i in 0..5000 {
            let a: u8 = random();
            let b: u8 = random();
            let c: bool = random();
            let d: bool = false;

            let msg = format!("For {:#04x} {:?}", opcode, op);
            let regr = format!("run_strop_monadic({:?}, {:#04x}, {}, {})", op, a, c, d);
            let truth = run_mos6502(opcode, a, 0xff, c, d);
            let strop = run_strop_monadic(op, a, c, d);

            assert!(
                truth.0 == strop.0,
                "{}, run {} and check accumulator == {:#04x}",
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
                "{}, run {} and check accumulator == {:#04x}",
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

    pub fn fuzz_test() {
        fuzz_monadic(LeftShiftArithmetic, 0x0a);
        fuzz_monadic(RightShiftLogical, 0x4a);
        fuzz_monadic(RotateLeftThruCarry, 0x2a);
        fuzz_monadic(RotateRightThruCarry, 0x6a);
        fuzz_dyadic(AddWithCarry, 0x69);
        fuzz_dyadic(And, 0x29);
        // Not testing BitCompare (the BIT opcode) here because there is no immediate mode
        fuzz_dyadic(ExclusiveOr, 0x49);
        fuzz_dyadic(Or, 0x09);
        fuzz_dyadic(SubtractWithCarry, 0xe9);
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
                "{}, run {} and check accumulator == {:#04x}",
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
        fuzz_dyadic(SubtractWithCarry, 0xe9);
        fuzz_dyadic(And, 0x29);
    }
}
