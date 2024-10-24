use bitmatch::bitmatch;

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
    R11,
    R12,
    Sp,
    Lr,
    Pc,
}

fn reg( r: &u32) -> Register {
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
        Register::R11,
        Register::R12,
        Register::Sp,
        Register::Lr,
        Register::Pc,
    ][*r as usize]
}

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
    LoadRegisterList,
    StoreRegisterList,
    BranchOffset,
    SwiNumber,
    Fixup
}

macro_rules! store_register_list {
    ($expr:expr) => {
        if $expr == Bitfield::StoreRegisterList {
            return Some((0, 16));
        }
    };
}

macro_rules! load_register_list {
    ($expr:expr) => {
        if $expr == Bitfield::LoadRegisterList {
            return Some((0, 16));
        }
    };
}

macro_rules! fixup {
    ($expr:expr, $field:expr) => {
        if $expr == Bitfield::Fixup {
            return Some($field);
        }
    };
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
        if $expr == Bitfield::Cond {
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

macro_rules! branch_offset {
    ($expr:expr, $field:expr) => {
        if $expr == Bitfield::BranchOffset {
            return Some($field);
        }
    };
}

macro_rules! swi_number {
    ($expr:expr, $field:expr) => {
        if $expr == Bitfield::SwiNumber {
            return Some($field);
        }
    };
}

fn branch_exchange(f: Bitfield) -> Option<(u32, u32)> {
    rn!(f, 0);
    cond!(f);
    None
}

fn alu(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    if word & 0x0fc0_00f0 == 0x0000_0090 {
        return multiply(word, f);
    }

    #[cfg(test)]
    assert!(!format!("{}", crate::armv4t::Insn(word)).starts_with("mul"));

    #[cfg(test)]
    assert!(!format!("{}", crate::armv4t::Insn(word)).starts_with("mla"));

    assert!(0x004000b4 != word);

    if word & 0x0ffffff0 == 0x012fff10 {
        return branch_exchange(f);
    }

    if word & 0x0f800000 == 0x01000000 {
        // psr transfer instructions
        // These are not supported by strop
        fixup!(f, (0, 22));
        return None;
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
            fixup!(f, (0, 5));
        }
    };
    cond!(f);
    None
}

fn multiply(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    if word & 0x0000_f000 != 0 {
        // Rn has to be zero
        fixup!(f, (0, 0));
    }
    set_condition_codes!(f, 20);
    rd!(f, 16);
    rn!(f, 12);
    rs!(f, 8);
    rm!(f, 0);
    cond!(f);
    None
}

fn single_data_swap(f: Bitfield) -> Option<(u32, u32)> {
    rn!(f, 16);
    rd!(f, 12);
    rm!(f, 0);
    cond!(f);
    None
}

fn halfword_data_transfer(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    let immediate = word & 0x01<< 22 != 0;
    rn!(f, 16);
    rd!(f, 12);

    if !immediate {
        rm!(f, 0);
    }

    cond!(f);
    None
}

fn single_data_transfer(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    // ldrh/strh/ldrsb/ldrsh
    let immediate = word & 0x0040_0000 != 0;
    rn!(f,16);
    rd!(f,12);

    if !immediate {
        rm!(f, 0);
    };
    cond!(f);
    None
}

fn block_data_transfer(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    rn!(f, 16);
    if word & 1 << 20 == 0 {
        store_register_list!(f);
    } else {
        load_register_list!(f);
    }
    cond!(f);
    None
}

fn branch(f: Bitfield) -> Option<(u32, u32)> {
    branch_offset!(f, (0,16));
    cond!(f);
    None
}

fn software_interrupt(f: Bitfield) -> Option<(u32, u32)> {
    swi_number!(f, (0,24));
    cond!(f);
    None
}

fn get_bitfield(word: u32, f: Bitfield) -> Option<(u32, u32)> {
    cond!(f);
    match word >> 24 & 0b1111 {
        0b0001 if word & 0x0fb0_0ff0 == 0x0100_0090 => single_data_swap(f),
        0b0000..=0b0001 if word & 0x0c00_0f90 == 0x0000_0090 => halfword_data_transfer(word,f),
        0b0100..=0b0111 if word & 0x0c00_0000 == 0x0400_0000 => single_data_transfer(word, f),
        0b0000..=0b0011 => alu(word, f),
        0b1000..=0b1001 => block_data_transfer(word, f),
        0b1010..=0b1011 => branch(f),
        0b1111 => software_interrupt(f),
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
                Register::R11,
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

    /// Resets the bitfield. This means that the bitfield, and all bits to the right, are set to 0,
    /// and the bits to the left of the bitfield are incremented.
    pub fn reset_bitfield(&mut self, field: (u32, u32)) {
        let (x, width) = field;
        let mask = (1u32 << (x + width)) - 1;
        self.0 |= mask;
        self.0 += 1;
    }

    /// If the instruction does not encode a valid ARMv4T instruction, then this method will change
    /// it to the numerically next valid instruction.
    #[bitmatch]
    pub fn fixup(&mut self) {
        #[bitmatch]
        fn f(i: &mut crate::armv4t::Insn) -> bool {
            fn bump16(i: &mut crate::armv4t::Insn) -> bool {
                i.0 |= 0xf;
                i.0 += 1;
                true
            }

            #[allow(unused_variables)]
            #[bitmatch]
            match i.0 {
                // strh. the bits marked zzzz should be zero
                "????_000_pu1w0_nnnn_dddd_zzzz_1??1_mmmm" => {
                    if z != 0 {
                        bump16(i)
                    } else {
                        false
                    }
                }

                // umull(s) and umlal(s). not available on armv4t.
                "????_0000_1???_hhhh_llll_mmmm_1001_nnnn" => {
                    bump16(i)
                }
                
                _ => false

            }
        }
        loop {
            if !f(self) {
                break;
            }
        }
    }

    #[bitmatch]
    fn uses(&self) -> (Vec<Register>, Vec<Register>) {
        #[bitmatch]
        fn f(word: u32) -> (Vec<u32>, Vec<u32>) {
            #[bitmatch]
            match word {

                // data transfer
                "????_000_????_?_nnnn_dddd_???????_0_mmmm" => (vec![n, m], vec![d]),
                "????_000_????_?_nnnn_dddd_ssss_0??_1_mmmm" => (vec![n, s, m], vec![d]),

                // multiplies
                "????_000000_0_?_dddd_0000_ssss_1001_mmmm" => (vec![m, s], vec![d]),
                "????_000000_1_?_dddd_nnnn_ssss_1001_mmmm" => (vec![m, s, n], vec![d]),

                // strh
                "????_000_???0?_nnnn_dddd_????_1??1_mmmm" => {
                    (vec![n, m], vec![d])
                }
                _ => (vec![], vec![])

            }
        }

        let (reads, writes) = f(self.0);

        (
            reads.iter().map(reg).collect(),
            writes.iter().map(reg).collect(),
        )
    }

    /// Returns a list of the registers which the instruction writes to
    pub fn writes(&self, register: Register) -> bool {
        self.uses().1.contains(&register)
    }

    /// Returns a list of the registers which the instruction reads from
    #[bitmatch]
    pub fn reads(&self, register: Register) -> bool {
        self.uses().0.contains(&register)
    }
}

#[cfg(test)]
mod test {
        use crate::armv4t::Insn;
        use crate::armv4t::isa::decode::Register;

    #[test]
    #[ignore]
    fn all_instructions_have_cond() {
        use crate::armv4t::isa::decode::Bitfield;
        use crate::armv4t::isa::decode::get_bitfield;

        // for i in 0..u32::MAX {
        for i in 0x00e00000..u32::MAX {
            let mut insn = Insn(i);

            let dasm = format!("{:?}", insn);
            println!("{dasm}");
            if dasm.starts_with("<illegal>") {
                assert!(get_bitfield(insn.0, Bitfield::Fixup).is_some());
                continue;
            }

            if let Some(rm) = insn.get_rm() {
                if rm == Register::R4 {
                    assert!(dasm.contains("r4"), "{:?}", insn);
                } else {
                    insn.increment_bitfield(Bitfield::Rm);
                }
            }
        }
    }

    #[test]
    fn r4() {
        use crate::Iterable;

        // let mut i = Insn::first();
        let mut i = Insn(0x00d00000);
        while i.step() {
            let dasm = format!("{:?}", i);

            if dasm.contains("r4") {
                if i.reads(Register::R4) {
                    continue;
                }

                if i.writes(Register::R4) {
                    continue;
                }

                panic!("{:?}", i);

            }
        }
    }
}
