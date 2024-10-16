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
    Nv,
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

/// An enumeration of the miscellaneous bitfields in the ARMv4T instruction
#[derive(Debug, PartialEq)]
pub enum Bitfield {
    Rm,
    Rd,
    RdHi,
    RdLo,
    SetConditionCodes,
    Cond,
    Opcode,
    Rn,
    Rs,
    Immediate,
    ShiftAmount,
}

macro_rules! shift_amount {
    ($expr:expr, $field:expr) => {
        if $expr == Bitfield::ShiftAmount {
            return Some($field);
        }
    };
}

macro_rules! immediate {
    ($expr:expr, $field:expr) => {
        if $expr == Bitfield::Immediate {
            return Some($field);
        }
    };
}

macro_rules! shift_type {
    ($expr:expr, $bit_number:expr) => {
        if $expr == Bitfield::SetConditionCodes {
            return Some(($bit_number, 2));
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
            return Some($e);
        }
    };
}

macro_rules! cond {
    ($expr:expr) => {
        if $expr == Bitfield::SetConditionCodes {
            return Some((28, 4));
        }
    };
}

macro_rules! rs {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rs {
            return Some(($offs, 4));
        }
    };
}

macro_rules! rm {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rm {
            return Some(($offs, 4));
        }
    };
}

macro_rules! rn {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rn {
            return Some(($offs, 4));
        }
    };
}

macro_rules! rd {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::Rd {
            return Some(($offs, 4));
        }
    };
}

macro_rules! rd_hi {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::RdHi {
            return Some(($offs, 4));
        }
    };
}

macro_rules! rd_lo {
    ($expr:expr, $offs:expr) => {
        if $expr == Bitfield::RdLo {
            return Some(($offs, 4));
        }
    };
}

fn branch_exchange(f: Bitfield) -> Option<(u32, u32)> {
    rn!(f, 0);
    cond!(f);
    None
}

fn alu(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    if word & 0x0ffffff0 == 0x012fff10 {
        return branch_exchange(f);
    }

    if word & 0x0f800000 == 0x01000000 {
        // psr transfer instructions
        todo!();
    }

    set_condition_codes!(f, 20);
    opcode!(f, (21, 4));

    rn!(f, 16);
    rd!(f, 12);

    if word & (1 << 25) != 0 {
        immediate!(f, (0, 8));
        shift_amount!(f, (8, 4));
    } else {
        shift_type!(f, 6);
        rm!(f, 0);

        if word & 0x10 == 0 {
            shift_amount!(f, (11, 8));
        } else {
            rs!(f, 8);
        }
    };
    cond!(f);
    None
}

fn multiply(f: Bitfield) -> Option<(u32, u32)> {
    set_condition_codes!(f, 20);
    rd!(f, 16);
    rn!(f, 12);
    rs!(f, 8);
    rm!(f, 0);
    None
}

fn multiply_long(f: Bitfield) -> Option<(u32, u32)> {
    rd_hi!(f, 16);
    rd_lo!(f, 12);
    rs!(f, 8);
    rm!(f, 0);
    cond!(f);
    None
}

fn get_bitfield(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    cond!(f);
    match word >> 24 & 0b1111 {
        0b0000..=0b0001 if extract(word, (4, 1)) == 0 || extract(word, (7, 1)) == 0 => alu(word, f),
        0b0010..=0b0011 => alu(word, f),
        0b0000 if extract(word, (7, 4)) == 9 => {
            if extract(word, (22, 2)) == 0 {
                multiply(f)
            } else {
                multiply_long(f)
            }
        }
        /*
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
        */
        _ => None,
    }
}

impl crate::armv4t::Insn {
    /// Returns the instruction's Condition field
    pub fn get_cond(&self) -> Option<Condition> {
        get_bitfield(self.0, Bitfield::Cond).map(|c| {
            [
                Condition::Eq,
                Condition::Ne,
                Condition::Cs,
                Condition::Cc,
                Condition::Mi,
                Condition::Pl,
                Condition::Vs,
                Condition::Vc,
                Condition::Hi,
                Condition::Ls,
                Condition::Ge,
                Condition::Lt,
                Condition::Gt,
                Condition::Le,
                Condition::Al,
                Condition::Nv,
            ][extract(self.0, c) as usize]
        })
    }

    fn get_register(&self, field: Bitfield) -> Option<Register> {
        get_bitfield(self.0, field).map(|f| {
            [
                Register::R0,
                Register::R1,
                Register::R2,
                Register::R3,
                Register::R4,
                Register::R5,
                Register::R6,
                Register::R7,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R12,
                Register::Sp,
                Register::Lr,
                Register::Pc,
            ][extract(self.0, f) as usize]
        })
    }

    /// Returns the instruction's Rm field
    pub fn get_rm(&self) -> Option<Register> {
        self.get_register(Bitfield::Rm)
    }

    /// Returns the instruction's Rd field
    pub fn get_rd(&self) -> Option<Register> {
        self.get_register(Bitfield::Rd)
    }

    /// Returns the instruction's Rs field
    pub fn get_rs(&self) -> Option<Register> {
        self.get_register(Bitfield::Rs)
    }

    /// Returns the instruction's Rn field
    pub fn get_rn(&self) -> Option<Register> {
        self.get_register(Bitfield::Rn)
    }

    /// Randomizes the bits inside the bitfield
    pub fn randomize_bitfield(&mut self, field: Bitfield) {
        use rand::Rng;

        let Some((x, width)) = get_bitfield(self.0, field) else {
            return;
        };
        let mask = ((1u32 << width) - 1) << x;
        let mut rng = rand::thread_rng();
        let random_bits = rng.gen::<u32>() & mask;
        self.0 = (self.0 & !mask) | random_bits;
    }

    /// Increments the bitfield. (Carry propagates out of the bitfield, so that if the bitfield
    /// rolls over, then the next bits to the left are incremented).
    pub fn increment_bitfield(&mut self, field: Bitfield) {
        let (x, _width) = get_bitfield(self.0, field).unwrap_or(
            // if the bitfield is not present in the instruction, then just increment the entire
            // instruction.
            (0, 0),
        );
        self.0 += 1 << x;
    }
}
