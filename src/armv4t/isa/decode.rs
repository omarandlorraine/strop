//! Module for decoding ARM instructions

/// A register
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl crate::armv4t::Insn {
    /// Returns list of registers written to or read by the instruction
    pub fn uses(&self) -> Vec<Register> {
        let dasm = format!("{self}");

        [
            (Register::R1, "r1"),
            (Register::R2, "r2"),
            (Register::R3, "r3"),
            (Register::R4, "r4"),
            (Register::R5, "r5"),
            (Register::R6, "r6"),
            (Register::R7, "r7"),
            (Register::R8, "r8"),
            (Register::R9, "r9"),
            (Register::R10, "r10"),
            (Register::R11, "r11"),
            (Register::R12, "r12"),
            (Register::Sp, "sp"),
            (Register::Lr, "lr"),
            (Register::Pc, "pc"),
            (Register::Sp, "r13"),
            (Register::Lr, "r14"),
            (Register::Pc, "r15"),
        ]
        .iter()
        .filter(|(_e, s)| dasm.contains(s))
        .map(|(e, _s)| *e)
        .collect()
    }
}
