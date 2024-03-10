//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::Compatibility;

use crate::SearchAlgorithm;

use crate::armv4t::instruction_set::Thumb;

/// A trait having methods for building search algorithms yielding ARM specific programs, such as
/// subroutines, IRQ handlers, FIQ handlers, etc.
pub trait ThumbSearch<S: SearchAlgorithm<Item = Thumb>> {
    /// returns an iterator yielding functions complying with the AAPCS32 calling conventions, and
    /// computing the provided functions.
    ///
    /// `func` should be a function returning an `Option<i32>`. For inputs where `func` returns
    /// `Some(x)`, the generated function returns `x`. But for inputs where `func` returns `None`,
    /// the behavior of the generated function is undefined.
    fn aapcs32(self, func: fn(i32, i32) -> Option<i32>) -> testers::Aapcs32<Self>
    where
        Self: SearchAlgorithm<Item = Thumb> + Sized,
    {
        testers::Aapcs32::new(self, func)
    }
}

impl<S: SearchAlgorithm<Item = Thumb>> ThumbSearch<S> for crate::StochasticSearch<Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>> ThumbSearch<S> for crate::BruteForceSearch<Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>> ThumbSearch<S> for crate::LengthLimitedSearch<S, Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>, C: Compatibility<instruction_set::Thumb>> ThumbSearch<S>
    for crate::CompatibilitySearch<S, Thumb, C>
{
}

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
