//! A module defining `Sequence<T>`.

use crate::Disassemble;
use crate::Encode;
use crate::Goto;
use crate::IterationResult;
use crate::Step;
use std::ops::{Index, IndexMut};

mod mutate;

/// `Sequence<T>` is a straight-line sequence of things, such as machine instructions or other
/// sequences.
///
/// This datatype is intended to represent a point in a search space, and so `impl`s
/// strop's `Random` and `Step` traits.  This means that strop can search across the search
/// space of things represented by the `Sequence<T>`.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Sequence<T>(Vec<T>);

impl<T: Step + crate::Encode<u8> + crate::Branch + crate::subroutine::ShouldReturn>
    crate::subroutine::ToSubroutine<T> for Sequence<T>
{
    fn to_subroutine(self) -> crate::Subroutine<T, Self> {
        crate::Subroutine::new(self)
    }
}

impl<Insn> AsRef<Sequence<Insn>> for Sequence<Insn> {
    fn as_ref(&self) -> &Sequence<Insn> {
        self
    }
}

impl<T> From<Vec<&Vec<T>>> for Sequence<T>
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

impl<T: Step> Sequence<T> {
    /// In a deterministic way compatible with the BruteForce search algorithm, mutates the
    /// Sequence at the offset in the given way.
    pub fn mut_at(&mut self, change: fn(&mut T) -> IterationResult, offset: usize) {
        if change(&mut self[offset]).is_err() {
            self[offset] = T::first();
            self.step_at(offset + 1);
        }
    }
}

impl<Insn: Step> crate::BruteforceSearch<Insn> for Sequence<Insn> {
    fn inner(&mut self) -> &mut dyn crate::BruteforceSearch<Insn> {
        unreachable!();
    }

    fn analyze_this(&self) -> Result<(), crate::StaticAnalysis<Insn>> {
        Ok(())
    }

    fn analyze(&mut self) -> Result<(), crate::StaticAnalysis<Insn>> {
        Ok(())
    }

    fn step(&mut self) {
        self.step_at(0);
    }

    fn apply(&mut self, static_analysis: &crate::StaticAnalysis<Insn>) {
        self.mut_at(static_analysis.advance, static_analysis.offset);
    }

    fn fixup(&mut self) {}
}

impl<T> Sequence<T> {
    /// Returns the index to the last element in the sequence
    pub fn last_instruction_offset(&self) -> usize {
        self.0.len() - 1
    }
    /// queries the item at offset `o` for static analysis
    pub fn sa<Insn>(
        &self,
        o: usize,
        sa: fn(&T) -> Option<crate::StaticAnalysis<Insn>>,
    ) -> Option<crate::StaticAnalysis<Insn>> {
        if let Some(mut r) = sa(&self.0[o]) {
            r.offset = o;
            return Some(r);
        }
        None
    }
}

impl<T> Index<usize> for Sequence<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Sequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> std::ops::Deref for Sequence<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Sequence<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T> IntoIterator for Sequence<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Disassemble for Sequence<T>
where
    T: Disassemble,
{
    fn dasm(&self) {
        for i in &self.0 {
            i.dasm();
        }
    }
}

impl<T, U> Encode<U> for Sequence<T>
where
    T: Encode<U>,
{
    fn encode(&self) -> Vec<U> {
        self.0.iter().flat_map(|i| i.encode()).collect()
    }
}

impl<T: Step> Sequence<T> {
    /// steps the sequence at the given offset
    pub fn step_at(&mut self, offs: usize) {
        let mut offset = offs;
        loop {
            if offset == self.0.len() {
                self.0.push(T::first());
                break;
            } else if self.0[offset].next().is_err() {
                self.0[offset] = T::first();
                offset += 1;
            } else {
                break;
            }
        }
    }
}

impl<T: Clone + Step> Step for Sequence<T> {
    fn first() -> Self {
        Self(vec![])
    }

    fn next(&mut self) -> IterationResult {
        self.step_at(0);
        Ok(())
    }
}

impl<SamplePoint: std::clone::Clone> Goto<SamplePoint> for Sequence<SamplePoint> {
    fn goto(&mut self, other: &[SamplePoint]) {
        self.0 = other.to_vec();
    }
}
