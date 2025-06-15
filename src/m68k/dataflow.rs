use crate::dataflow::DataFlow;
use crate::m68k::Insn;
use m68000::instruction::Operands;
use m68000::addressing_modes::AddressingMode;

pub enum Register {
    A0, D0,
    A1, D1,
    A2, D2,
    A3, D3,
    A4, D4,
    A5, D5,
    A6, D6,
    SP, D7,
}

impl DataFlow<Register> for Insn {
    fn reads(&self, datum: &Register) -> bool {
        match self.decode().0.operands {
            Operands::NoOperands => false,
            Operands::Immediate(_) => false,
            Operands::SizeEffectiveAddressImmediate(_size, effective_address, _immediate) => effective_address.reads(datum),
            Operands::EffectiveAddressCount(effective_address, count) => effective_address.reads(datum),
            Operands::EffectiveAddress(effective_address) => effective_address.reads(datum),
            Operands::SizeEffectiveAddress(_size, effective_address) => effective_address.reads(datum),
        }
    }

    fn writes(&self, datum: &Register) -> bool {
        match self.decode().0.operands {
            Operands::NoOperands => false,
            Operands::Immediate(_) => false,
            Operands::SizeEffectiveAddressImmediate(_size, effective_address, _immediate) => effective_address.writes(datum),
            Operands::EffectiveAddressCount(effective_address, count) => effective_address.writes(datum),
            Operands::EffectiveAddress(effective_address) => effective_address.writes(datum),
            Operands::SizeEffectiveAddress(_size, effective_address) => effective_address.writes(datum),
        }
    }

    fn sa(&self, _offset: usize)  -> crate::static_analysis::Fixup<Self> {
        todo!();
    }
}

impl DataFlow<Register> for AddressingMode {
    fn reads(&self, datum: &Register) -> bool {
        todo!();
    }
    fn writes(&self, datum: &Register) -> bool {
        todo!();
    }
    fn sa(&self, _offset: usize)  -> crate::static_analysis::Fixup<Self> {
        unreachable!();
    }
}
