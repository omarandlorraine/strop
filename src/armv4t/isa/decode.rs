/// The condition field in the instruction.
#[derive(Clone, Copy, Debug)]
pub enum Condition {
    /// equal
    Eq,
    /// not equal
    Ne,
    /// unsigned higher or same
    Cs,
    /// unsigned lower
    Cc,
    /// negative
    Mi,
    /// positive
    Pl,
    /// overflow
    Vs,
    /// no overflow
    Vc,
    /// unsigned higher
    Hi,
    /// unsigned lower or same
    Ls,
    /// greater or equal
    Ge,
    /// less than
    Lt,
    /// Greater than
    Gt,
    /// Less than or equal
    Le,
    /// Always
    Al,
    /// Never
    Nv
}

/// A register
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R12,
    Sp,
    Lr,
    Pc,
}


/// Operand2, for data processing/PSR transfer instructions
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operand2 {
    Shift(u8, Register),
    Shift2(Register, Register),
    Immediate(u8, u8),
}

/*
pub fn decode(word: u32) -> Operation {
    match word >> 24 & 0b1111 {
        0b0000...0b0001 if !bit!(word[4]) || !bit!(word[7]) => alu(word),
        0b0010...0b0011 => alu(word),
        0b0000 if bit!(word[23;22]) == 0 && bit!(word[7;4]) == 9 => multiply(word),
        0b0000 if bit!(word[23]) && bit!(word[7;4]) == 9 => multiply_long(word),
        0b0001 if !bit!(word[23]) && bit!(word[21;20]) == 0 && bit!(word[11;4]) == 9 => single_data_swap(word),
        0b0000...0b0001 if !bit!(word[22]) && bit!(word[11;7]) == 1 && bit!(word[6;5]) != 0 && bit!(word[4]) => halfword_data_transfer(word),
        0b0000...0b0001 if bit!(word[22]) && bit!(word[7]) && bit!(word[6;5]) != 0 && bit!(word[4]) => halfword_data_transfer(word),
        0b0100...0b0111 if !bit!(word[25]) || !bit!(word[4]) => single_data_transfer(word),
        0b1000...0b1001 => block_data_transfer(word),
        0b1010...0b1011 => branch(word),
        0b1100...0b1101 => coprocessor_data_transfer(word),
        0b1110 if !bit!(word[4]) => coprocessor_data_operation(word),
        0b1110 if bit!(word[4]) => coprocessor_register_transfer(word),
        0b1111 => software_interrupt(word),
        _ => undefined(),
    }
}


fn block_data_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    flags.pre = bit!(word[24]);
    flags.up = bit!(word[23]);
    let s = bit!(word[22]);
    flags.writeback = bit!(word[21]);
    flags.load = bit!(word[20]);

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;

    let list = bit!(word[15;0]) as u16;

    match (s, flags.load, list & 0x8000 != 0) {
        (false, _, _) => (),
        (true, true, true) => flags.psr = true,
        (true, false, true)
        | (true, true, false)
        | (true, false, false) => flags.user = true,
    }

    Operation::BlockDataTransfer(flags, registers, list)
}

fn branch(word: u32) -> Operation {
    let condition = bit!(word[31;28]);
    let link = bit!(word[24]);
    let offset = bit!(word[23;0]) as i32;

    Operation::Branch(condition.into(), link.into(), offset << 8 >> 6)
}

fn branch_exchange(word:u32) -> Operation {
    let mut registers = Registers::default();
    registers.n = bit!(word[3;0]) as u8;

    return Operation::BranchExchange(registers);
}

fn coprocessor_data_operation(word: u32) -> Operation {
    let coprocessor = bit!(word[11;8]) as u8;
    let opcode = bit!(word[23;20]) as u8;
    let information = bit!(word[7;5]) as u8;

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;
    registers.m = bit!(word[3;0]) as u8;

    Operation::CoprocessorDataOperation(coprocessor, opcode, information, registers)
}

fn coprocessor_data_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    flags.pre = bit!(word[24]);
    flags.up = bit!(word[23]);
    flags.transfer = bit!(word[22]);
    flags.writeback = bit!(word[21]);
    flags.load = bit!(word[20]);

    let coprocessor = bit!(word[11;8]) as u8;

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;

    let offset = bit!(word[7;0]);

    Operation::CoprocessorDataTransfer(flags, coprocessor, registers, offset << 2)
}

fn coprocessor_register_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    flags.load = bit!(word[20]);

    let coprocessor = bit!(word[11;8]) as u8;
    let opcode = bit!(word[23;21]) as u8;
    let information = bit!(word[7;5]) as u8;

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;
    registers.m = bit!(word[3;0]) as u8;

    Operation::CoprocessorRegisterTransfer(flags, coprocessor, opcode, information, registers)
}

fn halfword_data_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    flags.pre = bit!(word[24]);
    flags.up = bit!(word[23]);
    let immediate = bit!(word[22]);
    flags.writeback = bit!(word[21]);
    flags.load = bit!(word[20]);
    flags.signed = bit!(word[6]);
    flags.halfword = bit!(word[5]);

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;

    let offset = if immediate {
        let low = bit!(word[3;0]) as u8;
        let high = bit!(word[11;8]) as u8;
        let offset = high << 4 | low;

        Shift::Immediate { value: offset as u32 }
    } else {
        let m = bit!(word[3;0]) as u8;

        Shift::ImmediateShiftedRegister { amount: 0, shift: 0, m: m }
    };

    Operation::HalfwordDataTransfer(flags, registers, offset)
}

fn multiply(word: u32) -> Operation {
    let mut flags = MultiplyFlags::default();
    flags.accumulate = bit!(word[21]);
    flags.flags = bit!(word[20]);

    let mut registers = Registers::default();
    registers.d = bit!(word[19;16]) as u8;
    registers.n = bit!(word[15;12]) as u8;
    registers.s = bit!(word[11;8]) as u8;
    registers.m = bit!(word[3;0]) as u8;

    Operation::Multiply(flags, registers)
}

fn multiply_long(word: u32) -> Operation {
    let mut flags = MultiplyFlags::default();
    flags.unsigned = bit!(word[22]);
    flags.accumulate = bit!(word[21]);
    flags.flags = bit!(word[20]);

    let mut registers = Registers::default();
    registers.h = bit!(word[19;16]) as u8;
    registers.l = bit!(word[15;12]) as u8;
    registers.s = bit!(word[11;8]) as u8;
    registers.m = bit!(word[3;0]) as u8;

    Operation::MultiplyLong(flags, registers)
}

fn single_data_swap(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    flags.byte = bit!(word[22]);

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;
    registers.m = bit!(word[3;0]) as u8;

    Operation::SingleDataSwap(flags, registers)
}

fn single_data_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    let immediate = !bit!(word[25]);
    flags.pre = bit!(word[24]);
    flags.up = bit!(word[23]);
    flags.byte = bit!(word[22]);
    flags.writeback = bit!(word[21]);
    flags.load = bit!(word[20]);

    let mut registers = Registers::default();
    registers.n = bit!(word[19;16]) as u8;
    registers.d = bit!(word[15;12]) as u8;

    let offset = if immediate {
        let value = bit!(word[11;0]) as u32;

        Shift::Immediate { value: value }
    } else {
        let shift = bit!(word[6;5]) as u8;
        let m = bit!(word[3;0]) as u8;
        let amount = bit!(word[11;7]) as u8;

        Shift::ImmediateShiftedRegister { amount: amount, shift: shift, m: m }
    };

    Operation::SingleDataTransfer(flags, registers, offset)
}

fn software_interrupt(word: u32) -> Operation {
    let comment = bit!(word[23;0]);

    Operation::SoftwareInterrupt(comment)
}

fn status_transfer(word: u32) -> Operation {
    let mut flags = TransferFlags::default();
    let immediate = bit!(word[25]);
    flags.saved = bit!(word[22]);
    flags.load = !bit!(word[21]);

    let mask = bit!(word[19;16]) as u8;

    let mut registers = Registers::default();
    registers.d = bit!(word[15;12]) as u8;

    let value = if immediate {
        let rotate = bit!(word[11;8]) as u8;
        let immediate = bit!(word[7;0]) as u8;
        Shift::RotatedImmediate { rotation: rotate, immediate: immediate }
    } else {
        let m = bit!(word[3;0]) as u8;
        Shift::ImmediateShiftedRegister { amount: 0, shift: 0, m: m}
    };

    Operation::StatusTransfer(flags, mask, registers, value)
}

fn undefined() -> Operation {
    Operation::Undefined
}

fn decode(word: u32) -> Option<(u32, u32)> {
    match word >> 24 & 0b1111 {
        0b0000...0b0001 if !bit!(word[4]) || !bit!(word[7]) => alu(word),
        0b0010...0b0011 => alu(word),
        0b0000 if bit!(word[23;22]) == 0 && bit!(word[7;4]) == 9 => multiply(word),
        0b0000 if bit!(word[23]) && bit!(word[7;4]) == 9 => multiply_long(word),
        0b0001 if !bit!(word[23]) && bit!(word[21;20]) == 0 && bit!(word[11;4]) == 9 => single_data_swap(word),
        0b0000...0b0001 if !bit!(word[22]) && bit!(word[11;7]) == 1 && bit!(word[6;5]) != 0 && bit!(word[4]) => halfword_data_transfer(word),
        0b0000...0b0001 if bit!(word[22]) && bit!(word[7]) && bit!(word[6;5]) != 0 && bit!(word[4]) => halfword_data_transfer(word),
        0b0100...0b0111 if !bit!(word[25]) || !bit!(word[4]) => single_data_transfer(word),
        0b1000...0b1001 => block_data_transfer(word),
        0b1010...0b1011 => branch(word),
        0b1100...0b1101 => coprocessor_data_transfer(word),
        0b1110 if !bit!(word[4]) => coprocessor_data_operation(word),
        0b1110 if bit!(word[4]) => coprocessor_register_transfer(word),
        0b1111 => software_interrupt(word),
        _ => undefined(),
    }
}
*/

fn extract(word: u32, field: (u32, u32)) -> u32 {
    let (x, width) = field;
    let mask = (1u32 << width) - 1;
    (word >> x) & mask
}

#[derive(PartialEq)]
enum Bitfield {
    Rm,
    SetConditionCodes,
    Cond,
    Opcode,
    Rn,
    ShiftType,
}

macro_rules! shift_type {
    ($expr:expr, $bit_number:expr) => {
        if $expr == Bitfield::SetConditionCodes {
            return Some(($bit_number, 1));
        }
    };
}

macro_rules! set_condition_codes {
    ($expr:expr, $bit_number:expr) => {
        if $expr == Bitfield::SetConditionCodes {
            return Some(($bit_number, 1));
        }
    };
}

macro_rules! opcode {
    ($expr:expr, $e:expr) => {
        if $expr == Bitfield::Opcode {
            return Some(e);
        }
    };
}

macro_rules! cond {
    ($expr:expr) => {
        if $expr == Bitfield::S {
            return Some((28, 4));
        }
    };
}

macro_rules! rm {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rm {
            return Some((offs, 4));
        }
    };
}

macro_rules! rn {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rn {
            return Some((offs, 4));
        }
    };
}

macro_rules! rd {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rd {
            return Some((offs, 4));
        }
    };
}

fn alu(word: u32,f : Bitfield) -> Operation {
    set_condition_codes!(f, 20);
    opcode!(f, (21, 4));

    rn!(word,16);
    rd!(word,12);

    let operand = if bit!(word[25]) {
        let rotate = bit!(word[11;8]) as u8;
        let immediate = bit!(word[7;0]) as u8;
        Shift::RotatedImmediate { rotation: rotate, immediate: immediate }
    } else {
        shift_type!(f, (6, 2));
        rm!(f, 0);

        if word[4] & 0x10 == 0 {
            shift_amount!(f (11, 8));
            let amount = bit!(word[11;7]) as u8;
            Shift::ImmediateShiftedRegister { amount: amount, shift: shift, m: m }
        } else {
            rs!(word, 8);
        }
    };

    // 'br rn' instructions are equivalent to 'teq r15, rn, lsl r15'
    // without the S bit set.
    if let Shift::RegisterShiftedRegister { s: shift, .. } = operand {
        if shift == 15 && word & 0x0ffffff0 == 0x012fff10 {
            return branch_exchange(word);
        }
    }

    match (opcode, s) {
        (DataOp::Tst, false) if word & 0x0ffffff0 == 0x012fff10 => {
            return branch_exchange(word)
        }
        (DataOp::Tst, false)
        | (DataOp::Teq, false)
        | (DataOp::Cmp, false)
        | (DataOp::Cmn, false) => {
            return status_transfer(word);
        }
        _ => (),
    }

    Operation::Alu(opcode, s.into(), registers, operand)
}

fn get_bitfield(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    cond!(f);
    match word >> 24 & 0b1111 {
        0b0000..=0b0001 if extract(word,(4,1))==0 || extract(word,(7,1))==0 => alu(word),
        0b0010..=0b0011 => alu(word),
        0b0000 if extract(word,(23,22)) == 0 && extract(word,(7,4)) == 9 => multiply(word),
        0b0000 if extract(word,(23)) && extract(word,(7,4)) == 9 => multiply_long(word),
        0b0001 if !extract(word,(23)) && extract(word,(21,20)) == 0 && extract(word,(11,4)) == 9 => single_data_swap(word),
        0b0000..=0b0001 if !extract(word,(22)) && extract(word,(11,7)) == 1 && extract(word,(6,5)) != 0 && extract(word,(4)) => halfword_data_transfer(word),
        0b0000..=0b0001 if extract(word,(22)) && extract(word,(7)) && extract(word,(6,5)) != 0 && extract(word,(4)) => halfword_data_transfer(word),
        0b0100..=0b0111 if !extract(word,(25)) || !extract(word,(4)) => single_data_transfer(word),
        0b1000..=0b1001 => block_data_transfer(word),
        0b1010..=0b1011 => branch(word),
        0b1100..=0b1101 => coprocessor_data_transfer(word),
        0b1110 if !extract(word,(4)) => coprocessor_data_operation(word),
        0b1110 if extract(word,(4)) => coprocessor_register_transfer(word),
        0b1111 => software_interrupt(word),
        _ => undefined(),
    }
}
