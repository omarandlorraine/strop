use crate::Range;
use rand::prelude::SliceRandom;
use rand::Rng;

impl<T: std::cmp::PartialOrd + Ord + Copy> Range<T> for Vec<T> {
    fn random(&self) -> T {
        *self.choose(&mut rand::thread_rng()).unwrap()
    }

    fn next(&self, t: T) -> Option<T> {
        self.iter().filter(|&&x| x > t).min().copied()
    }

    fn check(&self, t: T) -> bool {
        self.contains(&t)
    }
}

impl<
        T: std::cmp::PartialOrd
            + Ord
            + Copy
            + rand::distributions::uniform::SampleUniform
            + std::ops::Add<i32, Output = T>,
    > Range<T> for std::ops::RangeInclusive<T>
{
    fn random(&self) -> T {
        rand::thread_rng().gen_range(self.clone())
    }

    fn next(&self, t: T) -> Option<T> {
        if t < *self.start() {
            Some(*self.start())
        } else if t > *self.end() {
            None
        } else {
            Some(t + 1)
        }
    }

    fn check(&self, t: T) -> bool {
        self.contains(&t)
    }
}
