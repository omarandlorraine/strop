use crate::instruction::Instruction;
use crate::snippets::Snippet;

pub struct Random<I: Instruction> {
    /// Iterator yielding random snippets of the given instruction type, and having any number of
    /// instructions up to the specified length.
    org: usize,
    max_length: usize,

    // for some reason I need this because an unused type parameter is a type error
    dummy: I,
}

impl<I: Instruction> Random<I> {
    pub fn new(org: usize, max_length: usize) -> Self {
        Self {
            org,
            max_length,
            dummy: I::new(),
        }
    }
}

impl<I: Instruction + std::fmt::Display> Iterator for Random<I> {
    type Item = Snippet<I>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Snippet::<I>::new_with_org_and_length(
            self.org,
            self.max_length,
        ))
    }
}
