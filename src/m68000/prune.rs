/// Module for ways to prune away instructions from the search
use crate::m68000::Insn;
use crate::ConstraintViolation;
use m68000::addressing_modes::AddressingMode;
use m68000::addressing_modes::BriefExtensionWord;
use m68000::instruction::Operands;

#[derive(Default, Debug)]
#[allow(missing_docs)]
pub struct Prune {
    pub data_registers: Vec<u8>,
    pub address_registers: Vec<u8>,

    pub offset_ranges: [std::ops::Range<i16>; 8],
    pub absolutes: Vec<u32>,
    pub immediate_values: Vec<u32>,
    pub pc_relative_range: std::ops::Range<i16>,
}

impl Prune {
    fn data_register_allowed(&self, reg_no: u8) -> bool {
        self.data_registers.contains(&reg_no)
    }
    fn address_register_allowed(&self, reg_no: u8) -> bool {
        self.address_registers.contains(&reg_no)
    }
    fn address_register_offset_allowed(&self, reg_no: u8, offset: i16) -> bool {
        self.offset_ranges[reg_no as usize].contains(&offset)
    }
    fn addressing_mode_allowed(&self, am: AddressingMode) -> bool {
        match am {
            AddressingMode::Drd(reg_no) => self.data_register_allowed(reg_no),
            AddressingMode::Ard(reg_no)
            | AddressingMode::Ari(reg_no)
            | AddressingMode::Ariwpo(reg_no)
            | AddressingMode::Ariwpr(reg_no)
            | AddressingMode::Ariwi8(reg_no, _) => self.address_register_allowed(reg_no),
            AddressingMode::Ariwd(reg_no, offset) => {
                self.address_register_offset_allowed(reg_no, offset)
            }
            AddressingMode::AbsShort(address) => {
                let address32 = address as u32;
                self.absolutes.contains(&address32)
            }
            AddressingMode::AbsLong(address32) => self.absolutes.contains(&address32),
            AddressingMode::Pciwd(_, disp) => self.pc_relative_range.contains(&disp),
            AddressingMode::Pciwi8(_, BriefExtensionWord(_)) => false,
            AddressingMode::Immediate(imm) => self.immediate_values.contains(&imm),
        }
    }
    fn immediate_allowed(&self, imm: u32) -> bool {
        self.immediate_values.contains(&imm)
    }
}

impl crate::Prune<Insn> for Prune {
    fn prune(&self, t: &Insn) -> ConstraintViolation<Insn> {
        if match t.decode().0.operands {
            Operands::Immediate(imm) => !self.immediate_allowed(imm.into()),
            Operands::SizeEffectiveAddressImmediate(_, am, imm) => {
                !self.addressing_mode_allowed(am) || !self.immediate_allowed(imm)
            }
            Operands::EffectiveAddressCount(am, _) => !self.addressing_mode_allowed(am),
            Operands::RegisterDirectionSizeRegisterDisplacement(d, _, _, a, offset) => {
                !self.data_register_allowed(d)
                    || !self.address_register_allowed(a)
                    || !self.address_register_offset_allowed(a, offset)
            }
            Operands::SizeEffectiveAddressEffectiveAddress(_, operand1, operand2) => {
                !self.addressing_mode_allowed(operand1) || !self.addressing_mode_allowed(operand2)
            }
            Operands::SizeRegisterEffectiveAddress(_, reg, ea) => {
                !self.address_register_allowed(reg) || !self.addressing_mode_allowed(ea)
            }
            Operands::SizeEffectiveAddress(_, ea) => !self.addressing_mode_allowed(ea),
            Operands::EffectiveAddress(ea) => !self.addressing_mode_allowed(ea),
            Operands::Register(r) => !self.data_register_allowed(r),
            Operands::OpmodeRegister(_, r) => !self.data_register_allowed(r),
            Operands::RegisterSizeEffectiveAddress(reg, _, ea)
            | Operands::RegisterEffectiveAddress(reg, ea) => {
                !self.data_register_allowed(reg) || !self.addressing_mode_allowed(ea)
            }
            Operands::DirectionSizeEffectiveAddressList(_, _, _, _) => true, // todo
            Operands::Vector(_) => true,                                     // todo
            Operands::RegisterDisplacement(reg, _disp) => !self.address_register_allowed(reg),
            Operands::DirectionRegister(_, reg) => !self.address_register_allowed(reg),
            Operands::NoOperands => false,
            Operands::DataSizeEffectiveAddress(_, _, ea) => !self.addressing_mode_allowed(ea),
            Operands::ConditionEffectiveAddress(_, ea) => !self.addressing_mode_allowed(ea),
            Operands::ConditionRegisterDisplacement(_, reg, _) => !self.data_register_allowed(reg),
            Operands::Displacement(_) => true,             // todo
            Operands::ConditionDisplacement(_, _) => true, // todo
            Operands::RegisterData(reg, _) => !self.data_register_allowed(reg),
            Operands::RegisterDirectionSizeEffectiveAddress(reg, _, _, ea) => {
                !self.data_register_allowed(reg) || !self.addressing_mode_allowed(ea)
            }
            Operands::RegisterSizeModeRegister(r1, _, _, r2) => {
                !self.data_register_allowed(r1) || !self.data_register_allowed(r2)
            }
            Operands::RegisterSizeRegister(a1, _, a2) => {
                !self.address_register_allowed(a1) || !self.address_register_allowed(a2)
            }
            Operands::RegisterOpmodeRegister(
                d0,
                m68000::instruction::Direction::ExchangeDataAddress,
                a1,
            ) => !self.data_register_allowed(d0) || !self.address_register_allowed(a1),
            Operands::RegisterOpmodeRegister(
                d0,
                m68000::instruction::Direction::ExchangeAddress,
                d1,
            ) => !self.address_register_allowed(d0) || !self.address_register_allowed(d1),
            Operands::RegisterOpmodeRegister(
                d0,
                m68000::instruction::Direction::ExchangeData,
                d1,
            ) => !self.data_register_allowed(d0) || !self.data_register_allowed(d1),
            Operands::RotationDirectionSizeModeRegister(_, _, _, _, d) => {
                !self.data_register_allowed(d)
            }
            Operands::DirectionEffectiveAddress(_, ea) => !self.addressing_mode_allowed(ea),
            _ => panic!("I don't know how to prune {:?}", t),
        } {
            return t.bump_opcode();
        }
        ConstraintViolation::Ok
    }
}

impl<Prune: crate::Prune<Self>> crate::PrunedSearch<Prune> for Insn {
    fn first() -> Self {
        Self::default()
    }

    fn pruned_step(&mut self, prune: &Prune) -> bool {
        use crate::Iterable;
        loop {
            self.step();
            match prune.prune(self) {
                ConstraintViolation::Ok => break true,
                ConstraintViolation::Violation => break false,
                ConstraintViolation::ReplaceWith(me) => *self = me,
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn nothing_allowed() {
        use super::Insn;
        use crate::m68000::prune::Prune;
        use crate::PrunedSearch;

        let prune = Prune::default();

        let mut i = Insn::default();

        while i.pruned_step(&prune) {
            println!("{:?}", i);
        }
    }
}
