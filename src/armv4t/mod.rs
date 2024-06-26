//! Backend targeting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod linkages;
pub mod testers;

#[cfg(test)]
mod test {
    #[test]
    fn all_instructions_can_be_executed() {
        use crate::armv4t::emulators::ArmV4T;
        use crate::armv4t::instruction_set::Thumb;
        use crate::BruteForceSearch;
        use crate::DummyFixup;
        use crate::Emulator;
        use crate::SearchAlgorithm;

        for candidate in BruteForceSearch::<Thumb, DummyFixup>::new(DummyFixup).iter() {
            if candidate.length() > 1 {
                break; //TODO
            }
            ArmV4T::default().run(0x2000, &candidate);
            candidate.disassemble("test");
        }
    }
}
