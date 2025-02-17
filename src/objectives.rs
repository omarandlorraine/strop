//! This module has a few objective functions in it which you might want to optimize for

use crate::Objective;
use crate::Sequence;

/// Objective function for optimizing for size
#[derive(Debug, Default)]
pub struct Short;

impl<I> Objective<Sequence<I>> for Short
where
    I: crate::Encode<u8>,
{
    fn score(&self, instruction_sequence: &Sequence<I>) -> f64 {
        use crate::Encode;
        instruction_sequence.encode().len() as f64
    }
}
