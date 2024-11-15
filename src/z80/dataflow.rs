use crate::z80::Insn;
/// Module containing functions which perform dataflow analysis on a `Sequence<Insn>`. This may be
/// used to narrow the search space
use dez80::instruction::InstructionType;
use dez80::instruction::Operand;
use dez80::register::RegisterPairType;
use dez80::register::SingleRegisterType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Register {
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
}

impl Register {
    pub fn all() -> Vec<Self> {
        vec![
            Register::A,
            Register::B,
            Register::C,
            Register::D,
            Register::E,
            Register::H,
            Register::L,
            Register::I,
            Register::Ixh,
            Register::Ixl,
            Register::Iyh,
            Register::Iyl,
        ]
    }

    pub fn rp(&self, r: RegisterPairType) -> bool {
        match r {
            RegisterPairType::AF => self == &Self::A,
            RegisterPairType::BC => matches!(self, Self::B | Self::C),
            RegisterPairType::DE => matches!(self, Self::D | Self::E),
            RegisterPairType::HL => matches!(self, Self::H | Self::L),
            RegisterPairType::AF_ => self == &Self::A,
            RegisterPairType::BC_ => matches!(self, Self::B | Self::C),
            RegisterPairType::DE_ => matches!(self, Self::D | Self::E),
            RegisterPairType::HL_ => matches!(self, Self::H | Self::L),
            RegisterPairType::IX => matches!(self, Self::Ixh | Self::Ixl),
            RegisterPairType::IY => matches!(self, Self::Iyh | Self::Iyl),
            RegisterPairType::PC => false,
            RegisterPairType::SP => false,
            RegisterPairType::IR => false,
            RegisterPairType::WZ => false,
        }
    }

    pub fn srt(&self, r: SingleRegisterType) -> bool {
        match self {
            Self::A => r == SingleRegisterType::A,
            Self::B => r == SingleRegisterType::B,
            Self::C => r == SingleRegisterType::C,
            Self::D => r == SingleRegisterType::D,
            Self::E => r == SingleRegisterType::E,
            Self::H => r == SingleRegisterType::H,
            Self::L => r == SingleRegisterType::L,
            Self::I => r == SingleRegisterType::I,
            Self::Ixh => r == SingleRegisterType::IXH,
            Self::Ixl => r == SingleRegisterType::IXL,
            Self::Iyh => r == SingleRegisterType::IYH,
            Self::Iyl => r == SingleRegisterType::IYL,
        }
    }

    pub fn check(&self, op: Option<Operand>) -> bool {
        match op {
            None => true,
            Some(Operand::DoubletImmediate(_)) => false,
            Some(Operand::MemoryDirect(_)) => false,
            Some(Operand::MemoryIndirect(rp)) => self.rp(rp),
            Some(Operand::MemoryIndexed(rp, _)) => self.rp(rp),
            Some(Operand::MemoryIndexedAndRegister(rp, _, r)) => self.rp(rp) | self.srt(r),
            Some(Operand::MemoryIndirectBit(rp, _)) => self.rp(rp),
            Some(Operand::MemoryIndexedBit(rp, _, _)) => self.rp(rp),
            Some(Operand::MemoryIndexedBitAndRegister(rp, _, _, r)) => self.rp(rp) | self.srt(r),
            Some(Operand::OctetImmediate(_)) => false,
            Some(Operand::OctetImplied(_)) => false,
            Some(Operand::PortDirect(_)) => false,
            Some(Operand::PortIndirect(_)) => false,
            Some(Operand::ProgramCounterRelative(_)) => false,
            Some(Operand::RegisterImplied(reg)) => self.srt(reg),
            Some(Operand::RegisterImpliedBit(reg, _)) => self.srt(reg),
            Some(Operand::RegisterPairImplied(rp)) => self.rp(rp),
        }
    }
}

impl crate::DataFlow<Register> for Insn {
    fn reads(&self, t: &Register) -> bool {
        let d = self.decode();

        if matches!(d.r#type, InstructionType::Nop | InstructionType::Ret(_)) {
            return false;
        }

        if matches!(d.r#type, InstructionType::Inc | InstructionType::Dec) {
            return t.check(d.destination);
        }
        t.check(d.source)
    }

    fn writes(&self, t: &Register) -> bool {
        let d = self.decode();

        if matches!(d.r#type, InstructionType::Nop | InstructionType::Ret(_)) {
            return false;
        }
        t.check(d.destination)
    }

    fn modify(&mut self) -> bool {
        self.next_opcode()
    }

    fn make_read(&mut self, t: &Register) -> bool {
        while !self.reads(t) {
            if !self.modify() {
                return false;
            }
        }
        true
    }

    fn make_write(&mut self, t: &Register) -> bool {
        while !self.writes(t) {
            if self.modify() {
                return false;
            }
        }
        true
    }
}
