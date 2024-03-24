//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod linkages;
pub mod testers;

use crate::Compatibility;
use crate::LinkageSearch;
use crate::Scalar;
use crate::SearchAlgorithm;

use crate::armv4t::instruction_set::Thumb;
use crate::armv4t::linkages::InterworkingSubroutine;

/// A trait having methods for building search algorithms yielding ARM specific programs, such as
/// subroutines, IRQ handlers, FIQ handlers, etc.
pub trait ThumbSearch {
    /// returns an iterator yielding functions complying with the AAPCS32 calling conventions, and
    /// computing the provided functions.
    ///
    /// `func` should be a function returning an `Option<i32>`. For inputs where `func` returns
    /// `Some(x)`, the generated function returns `x`. But for inputs where `func` returns `None`,
    /// the behavior of the generated function is undefined.
    fn aapcs32<T, U, V>(
        self,
        func: fn(T, U) -> Option<V>,
    ) -> testers::Aapcs32<LinkageSearch<Self, Thumb, InterworkingSubroutine>, T, U, V>
    where
        Self: SearchAlgorithm<Item = Thumb> + Sized,
        T: Scalar,
        U: Scalar,
        V: Scalar,
    {
        testers::Aapcs32::new(self.linkage(InterworkingSubroutine), func)
    }
}

impl ThumbSearch for crate::StochasticSearch<Thumb> {}
impl ThumbSearch for crate::BruteForceSearch<Thumb> {}
impl<S> ThumbSearch for crate::LengthLimitedSearch<S, Thumb> where S: SearchAlgorithm<Item = Thumb> {}
impl<S, C: Compatibility<instruction_set::Thumb>> ThumbSearch
    for crate::CompatibilitySearch<S, Thumb, C>
where
    S: SearchAlgorithm<Item = Thumb>,
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
