//! Module containing miscellaneous functions for testing callables
use crate::Callable;
use crate::StropError;
use rand;

/// Returns a few representative values for a given type
///
/// Useful for fuzz testing a Callable
pub trait Vals {
    /// Returns a few representative values
    fn vals() -> Vec<Self>
    where
        Self: std::marker::Sized;

    /// Returns a random value
    fn rand() -> Self;
}

impl Vals for bool {
    fn vals() -> Vec<Self> {
        vec![true, false]
    }

    fn rand() -> Self {
        rand::random()
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
}

/// Derives a simple test suite for the given callable.
pub fn quick_tests<
    InputParameters: Vals + Copy,
    ReturnValue: Vals,
    T: Callable<InputParameters, ReturnValue>,
>(
    callable: &T,
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
pub fn fuzz<P: Vals + Copy, R: Vals + std::cmp::PartialEq, T: Callable<P, R>, U: Callable<P, R>>(
    target_function: &T,
    candidate: &U,
    iterations: usize,
) -> Option<(P, R)> {
    for _ in 0..iterations {
        let i = P::rand();
        if let Ok(r) = target_function.call(i) {
            if let Ok(s) = candidate.call(i) {
                if r != s {
                    return Some((i, r));
                }
            }
        }
    }
    None
}

/// Checks if a callable passes the test suite.
pub fn passes<P: Vals + Copy, R: Vals + std::cmp::PartialEq, T: Callable<P, R>>(
    callable: &T,
    suite: &Vec<(P, R)>,
) -> Result<bool, StropError> {
    for t in suite {
        match callable.call(t.0) {
            Err(e) => {
                // The function doesn't pass the test because of some error during execution
                return Err(e);
            }
            Ok(r) => {
                if r != t.1 {
                    // The function doesn't pass the test because it returned some (valid) value
                    // different from the expected one
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}
