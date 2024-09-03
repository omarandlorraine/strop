//! Module containing miscellaneous functions for testing callables
use crate::Callable;

/// Returns a few representative values for a given type
///
/// Useful for fuzz testing a Callable
pub trait Vals {
    /// Returns a few representative values
    fn vals() -> Vec<Self>
    where
        Self: std::marker::Sized;
}

impl Vals for bool {
    fn vals() -> Vec<Self> {
        vec![true, false]
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
}

/// Derives a simple test suite for the given callable.
pub fn quick_tests<
    InputParameters: Vals + Copy,
    ReturnValue: Vals,
    SamplePoint,
    S,
    T: Callable<SamplePoint, S, InputParameters, ReturnValue>,
>(
    callable: T,
) -> Vec<(InputParameters, ReturnValue)> {
    let mut v = vec![];
    for p in InputParameters::vals() {
        if let Ok(r) = callable.call(p) {
            v.push((p, r))
        }
    }
    v
}

/// Checks if a callable passes the test suite.
pub fn passes<P: Vals + Copy, R: Vals + std::cmp::PartialEq, I, S, T: Callable<I, S, P, R>>(
    callable: T,
suite: Vec<(P, R)> ) -> bool {
    for t in suite {
        match callable.call(t.0) {
            Err(_) => {
                // The function doesn't pass the test because of some error during execution
                return false;
            }
            Ok(r) => {
                if r != t.1 {
                    // The function doesn't pass the test because it returned some (valid) value
                    // different from the expected one
                    return false;
                }
            }
        }
    }
    true
}
