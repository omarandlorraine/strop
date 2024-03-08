//! Backend targetting the ARMv4 CPUs (for example, the ARM7TDMI)

pub mod emulators;
pub mod instruction_set;
pub mod testers;

use crate::SearchAlgorithm;
use crate::BruteForceSearch;
use crate::Compatibility;
use crate::Candidate;
use crate::Fitness;

use crate::armv4t::instruction_set::Thumb;

pub struct ThumbSearch<S: SearchAlgorithm<Item = Thumb>>(S);

pub trait IntoThumbSearch<S: SearchAlgorithm<Item = Thumb>> {
    /// Builds and returns a [ThumbSearch] object.
    fn thumb( self) -> ThumbSearch<Self>
    where
        Self: Sized, Self: SearchAlgorithm<Item = Thumb>
    {
        ThumbSearch::<Self>::new(self)
    }
}

impl<S: SearchAlgorithm<Item = Thumb>> IntoThumbSearch<S> for crate::StochasticSearch<Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>> IntoThumbSearch<S> for crate::BruteForceSearch<Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>> IntoThumbSearch<S> for crate::LengthLimitedSearch<S, Thumb> {}
impl<S: SearchAlgorithm<Item = Thumb>, C: Compatibility<instruction_set::Thumb>> IntoThumbSearch<S> for crate::CompatibilitySearch<S, Thumb, C> {}

impl<S: SearchAlgorithm<Item = Thumb>> ThumbSearch<S> {

    /// Constructs a new ThumbSearch
    pub fn new(inner: S) -> Self {
        Self(inner)
    }


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

impl<S: SearchAlgorithm<Item = Thumb>> SearchAlgorithm for ThumbSearch<S> {
    type Item = Thumb;

    fn score(&mut self, score: f32) { self.0.score(score) }
    fn fitness(&mut self, candidate: &Candidate<Self::Item>) -> Fitness { self.0.fitness(candidate) }
    fn replace(&mut self, offset: usize, instruction: Option<Self::Item>) {self.0.replace(offset, instruction) }
    fn generate(&mut self) -> Option<Candidate<Self::Item>> {self.0.generate()}
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
