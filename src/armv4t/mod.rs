//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::BruteForceSearch;

use crate::armv4t::instruction_set::Thumb;

macro_rules! armimpl {
    ($t:ty) => {
        impl $t {
            /// returns an iterator yielding functions complying with the AAPCS32 calling conventions, and
            /// computing the provided functions.
            ///
            /// `func` should be a function returning an `Option<i32>`. For inputs where `func` returns
            /// `Some(x)`, the generated function returns `x`. But for inputs where `func` returns `None`,
            /// the behavior of the generated function is undefined.
            pub fn aapcs32(self, func: fn(i32, i32) -> Option<i32>) -> testers::Aapcs32<Self> {
                testers::Aapcs32::new(self, func)
            }
        }
    };
}

armimpl!(BruteForceSearch<Thumb>);

#[cfg(test)]
mod test {
    #[test]
    fn all_instructions_can_be_executed() {
        use crate::armv4t::emulators::ArmV4T;
        use crate::armv4t::Thumb;
        use crate::BruteForceSearch;
        use crate::Emulator;
        use crate::SearchAlgorithm;

        for candidate in BruteForceSearch::<Thumb>::new().iter() {
            if candidate.length() > 1 {
                break; //TODO
            }
            ArmV4T::default().run(0x2000, &candidate);
            candidate.disassemble();
        }
    }
}
