pub enum Fact {
    A, B, C,D,E,H,L,IX,IY,
    Sign, Zero, HalfCarry,
    Parity, AddSubtract,
    Carry
}

pub fn produces(seq: Sequence<Insn>, offs: usize, fact: Fact) {
}
