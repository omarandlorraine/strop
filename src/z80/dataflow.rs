use crate::dataflow::DataFlow;
use crate::z80::Insn;
use crate::StaticAnalysis;
use dez80::instruction::Operand;
use dez80::register::RegisterPairType;
use dez80::register::SingleRegisterType;
pub use dez80::register::Flag;

#[derive(Debug)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Ixh,
    Ixy,
    Iyh,
    Iyl,
}

impl DataFlow<Operand> for Insn {
    fn reads(&self, datum: &Operand) -> bool {
        let d = self.decode();

        d.source == Some(*datum)
    }

    fn writes(&self, datum: &Operand) -> bool {
        let d = self.decode();

        d.destination == Some(*datum)
    }

    fn sa(&self) -> StaticAnalysis<Self> {
        StaticAnalysis::<Self> {
            reason: "Dataflow",
            advance: Self::next_opcode,
            offset: 0,
        }
    }
}

impl DataFlow<Register> for Insn {
    fn reads(&self, datum: &Register) -> bool {
        match datum {
            Register::A => self.reads(&Operand::RegisterImplied(SingleRegisterType::A)),
            Register::B => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::B))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::BC))
                    || self.reads(&Operand::MemoryIndirect(RegisterPairType::BC))
            }
            Register::C => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::C))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::BC))
                    || self.reads(&Operand::MemoryIndirect(RegisterPairType::BC))
            }
            Register::D => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::D))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::DE))
                    || self.reads(&Operand::MemoryIndirect(RegisterPairType::DE))
            }
            Register::E => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::E))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::DE))
                    || self.reads(&Operand::MemoryIndirect(RegisterPairType::DE))
            }
            Register::H => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::H))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::HL))
            }
            Register::L => {
                self.reads(&Operand::RegisterImplied(SingleRegisterType::L))
                    || self.reads(&Operand::RegisterPairImplied(RegisterPairType::HL))
            }
            _ => todo!("{datum:?}"),
        }
    }

    fn writes(&self, datum: &Register) -> bool {
        match datum {
            Register::A => self.writes(&Operand::RegisterImplied(SingleRegisterType::A)),
            Register::B => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::B))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::BC))
            }
            Register::C => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::C))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::BC))
            }
            Register::D => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::D))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::DE))
            }
            Register::E => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::E))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::DE))
            }
            Register::H => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::H))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::HL))
            }
            Register::L => {
                self.writes(&Operand::RegisterImplied(SingleRegisterType::L))
                    || self.writes(&Operand::RegisterPairImplied(RegisterPairType::HL))
            }
            _ => todo!("{datum:?}"),
        }
    }

    fn sa(&self) -> StaticAnalysis<Self> {
        StaticAnalysis::<Self> {
            reason: "Dataflow",
            advance: Self::next_opcode,
            offset: 0,
        }
    }
}


impl DataFlow<Flag> for Insn {
    fn reads(&self, datum: &Flag) -> bool {
        use dez80::instruction::InstructionType;
        fn cond(datum: &Flag, cond: Option<dez80::instruction::Condition>) -> bool {
            match cond {
                None =>  false,
                Some(dez80::instruction::Condition::FlagSet(f)) => *datum == f,
                Some(dez80::instruction::Condition::FlagNotSet(f)) => *datum == f,
                Some(_) => false,
            }
        }
        match self.decode().r#type {
            InstructionType::Jr(cc) => cond(datum, cc),
            InstructionType::Jp(cc) => cond(datum, cc),
            InstructionType::Call(cc) => cond(datum, cc),
            _ => false,
        }
    }

    fn writes(&self, _datum: &Flag) -> bool {
        use dez80::instruction::InstructionType;
        match self.decode().r#type {
            InstructionType::Jr(_) => false,
            InstructionType::Jp(_) => false,
            InstructionType::Call(_) => false,
            _ => false,
        }
    }

    fn sa(&self) -> StaticAnalysis<Self> {
        StaticAnalysis::<Self> {
            reason: "Dataflow",
            advance: Self::next_opcode,
            offset: 0,
        }
    }
}


#[cfg(test)]
mod test {
    use crate::z80::Insn;
    use crate::dataflow::DataFlow;
    use super::Register;

    #[test]
    fn add_hl_bc() {
        let insn = Insn::new(&[0x09]);
        assert!(insn.reads(&Register::B));
        assert!(insn.reads(&Register::C));
        assert!(insn.writes(&Register::H));
        assert!(insn.writes(&Register::L));
    }
}
