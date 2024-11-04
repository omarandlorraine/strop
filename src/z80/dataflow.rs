use crate::z80::Insn;
/// Module containing functions which perform dataflow analysis on a `Sequence<Insn>`. This may be
/// used to narrow the search space
use crate::Sequence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fact {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    I,
    Ixh,
    Ixl,
    Iyh,
    Iyl,
    Carry,
    Negative,
    ParityOverflow,
    HalfCarry,
    Zero,
    Sign,
}

impl Fact {
    pub fn achzs(&self) -> bool {
        matches!(
            self,
            Fact::A | Fact::Carry | Fact::Negative | Fact::HalfCarry | Fact::Zero | Fact::Sign
        )
    }
    pub fn cnh(&self) -> bool {
        matches!(
            self,
            Fact::A | Fact::Negative | Fact::Carry | Fact::HalfCarry
        )
    }
    pub fn anh(&self) -> bool {
        matches!(self, Fact::A | Fact::Negative | Fact::HalfCarry)
    }
    pub fn acnh(&self) -> bool {
        matches!(
            self,
            Fact::A | Fact::Carry | Fact::Negative | Fact::HalfCarry
        )
    }
    pub fn acph(&self) -> bool {
        matches!(
            self,
            Fact::A | Fact::Carry | Fact::ParityOverflow | Fact::HalfCarry
        )
    }
    pub fn is_flag(&self) -> bool {
        matches!(
            self,
            Fact::Carry
                | Fact::Negative
                | Fact::ParityOverflow
                | Fact::HalfCarry
                | Fact::Zero
                | Fact::Sign
        )
    }
}

/// Checks if the sequence, at some point from the offset onwards, affects the given register or
/// flag
pub fn check_produces(seq: &Sequence<Insn>, offs: usize, fact: Fact) -> bool {
    seq.iter().skip(offs).any(|i| i.produces(fact))
}

/// Mutates the sequence, from the offset onwards, so that it affects the given register or flag
pub fn make_produce(seq: &mut Sequence<Insn>, offs: usize, fact: Fact) {
    while !check_produces(seq, offs, fact) {
        seq[offs].next_opcode();
    }
}
