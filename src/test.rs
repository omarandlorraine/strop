//! Module containing miscellaneous functions for testing callables
use crate::Callable;
use rand;

/// Returns a few representative values for a given type
///
/// Useful for fuzz testing a Callable
pub trait Vals: std::cmp::PartialEq + Copy + std::fmt::Debug {
    /// Returns a few representative values
    fn vals() -> Vec<Self>
    where
        Self: std::marker::Sized;

    /// Returns a random value
    fn rand() -> Self;

    /// Returns the difference between A and B
    fn error(self, other: Self) -> f64;
}

impl Vals for bool {
    fn vals() -> Vec<Self> {
        vec![true, false]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: bool) -> f64 {
        if self == other {
            1.0
        } else {
            0.0
        }
    }
}

impl Vals for u8 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..8 {
            v.push(1 << i);
            v.push(i);
            v.push(u8::MAX - i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl Vals for i8 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(i8::MAX - i);
            v.push(i8::MIN + i);
            v.push(-i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl Vals for i16 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(i16::MAX - i);
            v.push(i16::MIN + i);
            v.push(-i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl Vals for u16 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(u16::MAX - i);
            v.push(u16::MIN + i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl Vals for f32 {
    fn vals() -> Vec<Self> {
        vec![0.0, -1.0, 1.0, -0.5, 0.5]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self - other).abs().into()
    }
}

impl<A: Vals + Copy, B: Vals + Copy> Vals for (A, B) {
    fn vals() -> Vec<Self> {
        let mut v = vec![];
        for a in A::vals() {
            for b in B::vals() {
                v.push((a, b));
            }
        }
        v
    }

    fn rand() -> Self {
        (A::rand(), B::rand())
    }

    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1)
    }
}

impl<A: Vals + Copy, B: Vals + Copy, C: Vals + Copy> Vals for (A, B, C) {
    fn vals() -> Vec<Self> {
        let mut v = vec![];
        for a in A::vals() {
            for b in B::vals() {
                for c in C::vals() {
                    v.push((a, b, c));
                }
            }
        }
        v
    }

    fn rand() -> Self {
        (A::rand(), B::rand(), C::rand())
    }

    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1) + self.2.error(other.2)
    }
}

/// Holds test cases and their results.
#[derive(Clone, Debug, Default)]
pub struct TestSuite<InputParameters, ReturnValue>(Vec<(InputParameters, ReturnValue)>);

impl<InputParameters: Vals, ReturnValue: Vals> TestSuite<InputParameters, ReturnValue> {
    /// Derives a simple test suite for the given callable, and constructs a test suite from the same.
    pub fn generate<T: Callable<InputParameters, ReturnValue>>(callable: &T) -> Self {
        let mut v = vec![];
        for p in InputParameters::vals() {
            if let Ok(r) = callable.call(p) {
                v.push((p, r))
            }
        }
        Self(v)
    }

    /// Checks if a callable passes the test suite.
    pub fn passes<T: Callable<InputParameters, ReturnValue>>(
        &self,
        callable: &T,
    ) -> crate::RunResult<bool> {
        for t in self.0.iter() {
            let r = callable.call(t.0)?;
            if r != t.1 {
                // The function doesn't pass the test because it returned some (valid) value
                // different from the expected one
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Fuzz tests two callables against eachother by calling each with random parameters. If any
    /// parameters are found to give differing return values for both callables, then this is a new
    /// failing test and it is added to the test suite.
    ///
    /// If such a test is found, then the method returns `false`, signifying that the test did not
    /// pass. Otherwise, the method returns `true`.
    pub fn fuzz<
        T: Callable<InputParameters, ReturnValue>,
        U: Callable<InputParameters, ReturnValue>,
    >(
        &mut self,
        target_function: &T,
        candidate: &U,
        iterations: usize,
    ) -> bool {
        for _ in 0..iterations {
            let i = InputParameters::rand();
            if let Ok(r) = target_function.call(i) {
                if let Ok(s) = candidate.call(i) {
                    if r != s {
                        self.0.push((i, r));
                        return false;
                    }
                }
            }
        }
        true
    }

    /// First checks that the candidate function passes the known test cases. If not, fuzz tests
    /// the candidate against the target function by calling the `fuzz` method.
    ///
    /// The purpose here is to be able to return faster if the candidate function is unlikely to
    /// pass the test cases.
    pub fn checked_fuzz<
        T: Callable<InputParameters, ReturnValue>,
        U: Callable<InputParameters, ReturnValue>,
    >(
        &mut self,
        target_function: &T,
        candidate: &U,
        iterations: usize,
    ) -> bool {
        match self.passes(target_function) {
            Err(_) => false,
            Ok(false) => false,
            Ok(true) => self.fuzz(target_function, candidate, iterations),
        }
    }

    /// Computes the hamming distance between the callable's results and the results already in the
    /// test suite.
    pub fn score<T: Callable<InputParameters, ReturnValue>>(&self, callable: &T) -> f64 {
        let mut result = 0.0f64;
        for t in self.0.iter() {
            match callable.call(t.0) {
                Err(_) => {
                    // The function doesn't pass the test because of some error during execution
                    return f64::MAX;
                }
                Ok(r) => {
                    result += r.error(t.1);
                }
            }
        }
        result
    }
}
