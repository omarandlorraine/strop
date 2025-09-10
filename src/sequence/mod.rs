//! A module defining `Sequence<T>`.

use crate::Disassemble;
use crate::Encode;
use crate::IterationResult;
use crate::static_analysis::Fixup;
use std::ops::{Index, IndexMut};
use crate::search::Instruction;

/// `Sequence<Insn>` is a straight-line sequence of machine instructions.
///
/// This datatype is intended to represent a point in a search space, and so its methods
/// convenience walking around a search space, facilitating exhaustive and stochastic search
/// algorithms.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Sequence<Insn: Instruction>(Vec<Insn>);

impl<T: Instruction> From<Vec<&Vec<T>>> for Sequence<T>
where
    T: Clone,
{
    fn from(v: Vec<&Vec<T>>) -> Self {
        let mut r: Vec<T> = vec![];

        for s in v {
            for i in s {
                r.push(i.clone());
            }
        }

        Self(r)
    }
}

impl<T: Instruction> Sequence<T> {
    /// In a deterministic way compatible with the BruteForce search algorithm, mutates the
    /// Sequence at the offset in the given way.
    pub fn mut_at(&mut self, change: fn(&mut T) -> IterationResult, offset: usize) {
        if change(&mut self[offset]).is_err() {
            self[offset] = T::first();
            self.step_at(offset + 1);
        }
    }

    /// Applies the fixup to the code sequence
    pub fn apply_fixup(&mut self, fixup: &Fixup<T>) {
        self.mut_at(fixup.advance, fixup.offset);
    }

    /// steps the sequence at the given offset
    fn step_at(&mut self, offs: usize) {
        let mut offset = offs;
        loop {
            if offset == self.0.len() {
                self.0.push(T::first());
                break;
            } else if self.0[offset].increment().is_err() {
                self.0[offset] = T::first();
                offset += 1;
            } else {
                break;
            }
        }
    }

    pub fn next(&mut self) -> IterationResult {
        self.step_at(0);
        Ok(())
    }
}

impl<T: Instruction> Sequence<T> {
    /// Returns the index to the last element in the sequence
    pub fn last_instruction_offset(&self) -> usize {
        self.0.len() - 1
    }
}

impl<T: Instruction> Index<usize> for Sequence<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Instruction> IndexMut<usize> for Sequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T: Instruction> std::ops::Deref for Sequence<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Instruction> std::ops::DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T: Instruction> IntoIterator for Sequence<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Instruction> Disassemble for Sequence<T>
where
    T: Disassemble,
{
    fn dasm(&self) {
        for i in &self.0 {
            i.dasm();
        }
    }
}

impl<T: Instruction, U> Encode<U> for Sequence<T>
where
    T: Encode<U>,
{
    fn encode(&self) -> Vec<U> {
        self.0.iter().flat_map(|i| i.encode()).collect()
    }
}
