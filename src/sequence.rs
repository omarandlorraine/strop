//! A module defining `Sequence<T>`.

use crate::Disassemble;
use crate::Encode;
use crate::Goto;
use crate::Iterable;
use crate::Random;
use std::ops::{Index, IndexMut};

/// `Sequence<T>` is a straight-line sequence of things, such as machine instructions or other
/// sequences.
///
/// This datatype is intended to represent a point in a search space, and so `impl`s
/// strop's `Random` and `Iterable` traits.  This means that strop can search across the search
/// space of things represented by the `Sequence<T>`.
#[derive(Clone, Debug)]
pub struct Sequence<T>(Vec<T>);

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

impl<T: Iterable, U> crate::DataFlow<U> for Sequence<T>
where
    T: crate::DataFlow<U>,
{
    fn reads(&self, t: &U) -> bool {
        for i in &self.0 {
            if i.reads(t) {
                return true;
            }
            if i.writes(t) {
                return false;
            }
        }
        false
    }

    fn writes(&self, t: &U) -> bool {
        for i in &self.0 {
            if i.writes(t) {
                return true;
            }
        }
        false
    }

    fn make_read(&mut self, t: &U) -> bool {
        if !self.0[0].make_read(t) {
            self.step_at(0);
        }
        true
    }

    fn make_write(&mut self, t: &U) -> bool {
        if !self.0[0].make_write(t) {
            self.step_at(0);
        }
        true
    }
}

impl<T: Iterable> Sequence<T> {
    /// Returns the index to the last element in the sequence
    pub fn last_instruction_offset(&self) -> usize {
        self.0.len() - 1
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

impl<T> Sequence<T> {
    fn random_offset(&self) -> usize {
        use rand::Rng;
        rand::thread_rng().gen_range(0..self.0.len())
    }
}

impl<T: Iterable> Sequence<T> {
    /// steps the sequence at the given offset
    pub fn step_at(&mut self, offs: usize) {
        let mut offset = offs;
        loop {
            if offset == self.0.len() {
                self.0.push(T::first());
                break;
            } else if !self.0[offset].step() {
                self.0[offset] = T::first();
                offset += 1;
            } else {
                break;
            }
        }
    }
}

impl<T: Clone + Iterable> Iterable for Sequence<T> {
    fn first() -> Self {
        Self(vec![])
    }

    fn step(&mut self) -> bool {
        self.step_at(0);
        true
    }
}

impl<T: Clone + Random> Random for Sequence<T> {
    fn random() -> Self {
        Self(vec![])
    }

    fn step(&mut self) {
        use rand::Rng;
        let choice = rand::thread_rng().gen_range(0..5);

        match choice {
            0 => {
                // If the list of instructions contains at least one instruction, then delete one at
                // random.
                if !self.0.is_empty() {
                    let offset = self.random_offset();
                    self.0.remove(offset);
                }
            }
            1 => {
                // Insert a randomly generated instruction at a random location in the program.
                let offset = if self.0.is_empty() {
                    0
                } else {
                    self.random_offset()
                };
                self.0.insert(offset, T::random());
            }
            2 => {
                // If the program contains at least two instructions, then pick two at random and swap them
                // over.
                if self.0.len() > 2 {
                    let offset_a = self.random_offset();
                    let offset_b = self.random_offset();
                    self.0.swap(offset_a, offset_b);
                }
            }
            3 => {
                // If the list of instructions contains at least one instruction, then pick one at random
                // and swap it for something totally different.
                if !self.0.is_empty() {
                    let offset = self.random_offset();
                    self.0[offset] = T::random();
                }
            }
            4 => {
                // If the list of instructions contains at least one instruction, then pick one at random
                // and call its `mutate` method.
                if !self.0.is_empty() {
                    let offset = self.random_offset();
                    self.0[offset].step();
                }
            }
            _ => panic!(),
        }
    }
}

impl<SamplePoint: std::clone::Clone> Goto<SamplePoint> for Sequence<SamplePoint> {
    fn goto(&mut self, other: &[SamplePoint]) {
        self.0 = other.to_vec();
    }
}
