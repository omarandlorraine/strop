#![allow(missing_docs)]

#[derive(Clone, Copy, Default, Debug)]
pub enum ReadWrite {
    /// Leaves the datum alone
    #[default]
    N,
    /// Read-modify-write
    Rmw,
    /// Read
    R,
    ///Write,
    W,
}

impl ReadWrite {
    fn reads(&self) -> bool {
        matches!(self, ReadWrite::R | ReadWrite::Rmw)
    }
    fn writes(&self) -> bool {
        matches!(self, ReadWrite::W | ReadWrite::Rmw)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InstructionData {
    pub mnemonic: &'static str,
    pub opcode: u8,
    pub bytes: usize,
    pub cycles: usize,
    pub zero: ReadWrite,
    pub negative: ReadWrite,
    pub half_carry: ReadWrite,
    pub carry: ReadWrite,
    pub a: ReadWrite,
    pub b: ReadWrite,
    pub c: ReadWrite,
    pub d: ReadWrite,
    pub e: ReadWrite,
    pub h: ReadWrite,
    pub l: ReadWrite,
    pub iyl: ReadWrite,
    pub iyh: ReadWrite,
    pub ixl: ReadWrite,
    pub ixh: ReadWrite,
    pub r: ReadWrite,
    pub i: ReadWrite,
    pub sp: ReadWrite,

    pub operands: [&'static str; 3],
}

impl InstructionData {
    pub fn datum(&self, datum: &Datum) -> ReadWrite {
        match datum {
            Datum::Zero => self.zero,
            Datum::Negative => self.negative,
            Datum::HalfCarry => self.half_carry,
            Datum::Carry => self.carry,
            Datum::A => self.a,
            Datum::B => self.b,
            Datum::C => self.c,
            Datum::D => self.d,
            Datum::E => self.e,
            Datum::H => self.h,
            Datum::L => self.l,
            Datum::Iyl => self.iyl,
            Datum::Iyh => self.iyh,
            Datum::Ixl => self.ixh,
            Datum::Ixh => self.ixh,
            Datum::R => self.r,
            Datum::I => self.i,
            Datum::Sp => self.sp,
        }
    }
}

#[derive(Debug)]
pub enum Datum {
    Zero,
    Negative,
    HalfCarry,
    Carry,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Iyl,
    Iyh,
    Ixl,
    Ixh,
    R,
    I,
    Sp,
}

impl<Insn: super::X80> crate::dataflow::DataFlow<Datum> for Insn {
    fn reads(&self, datum: &Datum) -> bool {
        self.decode().datum(datum).reads()
    }

    fn writes(&self, datum: &Datum) -> bool {
        self.decode().datum(datum).writes()
    }

    fn sa(&self, offset: usize) -> crate::static_analysis::Fixup<Self> {
        crate::static_analysis::Fixup::new("Dataflow", Self::next_opcode, offset)
    }
}
