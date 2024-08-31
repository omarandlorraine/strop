//! A module defining `Sequence<T>`.

use crate::Encode;
use crate::Goto;
use crate::Iterable;
use crate::Random;
use std::ops::Index;

/// `Sequence<T>` is a straight-line sequence of things, such as machine instructions or other
/// sequences.
///
/// This datatype is intended to represent a point in a search space, and so `impl`s
/// strop's `Random` and `Iterable` traits.  This means that strop can search across the search
/// space of things represented by the `Sequence<T>`.
#[derive(Clone, Debug)]
pub struct Sequence<T>(Vec<T>);

// Implement the Index trait for read-only access.
impl<T> Index<usize> for Sequence<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
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

impl<T: Clone + Iterable> Iterable for Sequence<T> {
    fn first() -> Self {
        Self(vec![])
    }

    fn step(&mut self) -> bool {
        let mut offset = 0;
        loop {
            if offset == self.0.len() {
                self.0.push(T::first());
            } else if self.0[offset].step() {
                offset += 1;
            } else {
                return true;
            }
        }
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

impl<I: std::clone::Clone> Goto<I> for Sequence<I> {
    fn goto(&mut self, other: &[I]) {
        self.0 = other.to_vec();
    }
}
