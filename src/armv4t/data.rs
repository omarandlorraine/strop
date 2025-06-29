//! Dataflow analysis for ARM instructions.
//!
//! This module implements dataflow analysis for ARM instructions.
use crate::armv4t::Insn;
use crate::static_analysis::Fixup;

pub use unarm::args::Register;

/// Condition flags. Conditional instructions read from this, and instructions which set the
/// condition flags (such as `ands`) write to the condition flags.
#[derive(Debug)]
pub struct ConditionFlags;

impl crate::dataflow::DataFlow<ConditionFlags> for Insn {
    fn reads(&self, _datum: &ConditionFlags) -> bool {
        // reading from the Condition Flags means, the instruction is conditional.
        self.0 < 0xe000_0000
    }

    fn writes(&self, _datum: &ConditionFlags) -> bool {
        self.decode().updates_condition_flags()
    }

    fn sa(&self, offset: usize) -> Fixup<Self> {
        Fixup::new("ConditionDataflow", Self::next_opcode, offset)
    }
}

impl crate::dataflow::DataFlow<Register> for Insn {
    fn reads(&self, reg: &Register) -> bool {
        use unarm::args::Argument;

        let argl = self.decode().uses(&Default::default());
        for arg in argl {
            match arg {
                Argument::Reg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::ShiftReg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::OffsetReg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::CoReg(_) => unreachable!(),
                Argument::StatusReg(_) => continue,
                Argument::RegList(list) => {
                    if list.contains(*reg) {
                        return true;
                    }
                }
                Argument::StatusMask(s) => unreachable!("{self:?}, {s:?}"),
                Argument::Shift(_) => continue,
                Argument::ShiftImm(_) => continue,
                Argument::SImm(_) => continue,
                Argument::OffsetImm(_) => continue,
                Argument::BranchDest(_) => continue,
                Argument::CoOption(_) => unreachable!(),
                Argument::CoOpcode(_) => unreachable!(),
                Argument::CoprocNum(_) => unreachable!(),
                Argument::CpsrMode(_) => unreachable!(),
                Argument::CpsrFlags(_) => unreachable!(),
                Argument::Endian(_) => continue,
                Argument::UImm(_) => continue,
                Argument::SatImm(_) => continue,
                Argument::None => continue,
            }
        }
        false
    }

    fn writes(&self, reg: &Register) -> bool {
        use unarm::args::Argument;
        let argl = self.decode().defs(&Default::default());
        for arg in argl {
            match arg {
                Argument::Reg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::ShiftReg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::OffsetReg(r) => {
                    if r.reg == *reg {
                        return true;
                    }
                }
                Argument::CoReg(_) => unreachable!(),
                Argument::StatusReg(_) => continue,
                Argument::RegList(list) => {
                    if list.contains(*reg) {
                        return true;
                    }
                }
                Argument::StatusMask(s) => unreachable!("{self:?}, {s:?}"),
                Argument::Shift(_) => continue,
                Argument::ShiftImm(_) => continue,
                Argument::SImm(_) => continue,
                Argument::OffsetImm(_) => continue,
                Argument::BranchDest(_) => continue,
                Argument::CoOption(_) => unreachable!(),
                Argument::CoOpcode(_) => unreachable!(),
                Argument::CoprocNum(_) => unreachable!(),
                Argument::CpsrMode(_) => unreachable!(),
                Argument::CpsrFlags(_) => unreachable!(),
                Argument::Endian(_) => continue,
                Argument::UImm(_) => continue,
                Argument::SatImm(_) => continue,
                Argument::None => continue,
            }
        }
        false
    }

    fn sa(&self, offset: usize) -> Fixup<Self> {
        use crate::Step;
        Fixup::new("RegisterDataflow", Self::next, offset)
    }
}

#[cfg(test)]
mod test {
    use super::{ConditionFlags, Register};
    use crate::dataflow::DataFlow;
    #[test]
    fn validate() {
        use crate::Step;
        let mut i = crate::armv4t::Insn::default();

        loop {
            i.reads(&ConditionFlags);
            i.writes(&ConditionFlags);
            i.reads(&Register::R4);
            i.writes(&Register::R4);

            if i.next().is_err() {
                break;
            }
        }
    }
}
