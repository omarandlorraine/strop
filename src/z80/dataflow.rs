/// Module containing functions which perform dataflow analysis on a `Sequence<Insn>`. This may be
/// used to narrow the search space

use crate::Sequence;
use crate::z80::Insn;

#[derive(Clone, Copy)]
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

/// Checks if the sequence, at some point from the offset onwards, affects the given register or
/// flag
pub fn check_produces(seq: &Sequence<Insn>, offs: usize, fact: Fact) -> bool {
    seq.iter().skip(offs).any(|i| i.produces(fact))
}

/// Mutates the sequence, from the offset onwards, so that it affects the given register or flag
pub fn make_produce(seq: &mut Sequence<Insn>, offs: usize, fact: Fact) {
    use crate::IterableSequence;
    while !check_produces(seq, offs, fact) {
        seq.stride_at(offs);
    }
}
