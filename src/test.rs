//! A module implementing a basic fuzz-tester.
//!
//! This works by picking a few hand-picked values for each supported type and any number of random
//! values also. Then, by using a `Callable` as a reference, memoizes the results. These results
//! can be compared against what a putative `Callable` does too.

use crate::Callable;
use crate::RunResult;

/// For emulators, etc., which can have the function's parameters stuck in them as part of a
/// function call.
///
/// Implementations will take the value and put them in the emulated machine's register file, stack
/// frame or whatever else, in a manner consistent with its calling convention.
#[allow(missing_docs)]
pub trait TakeParameters {
    fn put_bool(&mut self, v: bool) -> RunResult<()>;
    fn put_i8(&mut self, v: i8) -> RunResult<()>;
    fn put_u8(&mut self, v: u8) -> RunResult<()>;
    fn put_i16(&mut self, v: i16) -> RunResult<()>;
    fn put_u16(&mut self, v: u16) -> RunResult<()>;
    fn put_i32(&mut self, v: i32) -> RunResult<()>;
    fn put_u32(&mut self, v: u32) -> RunResult<()>;
    fn put_f32(&mut self, v: f32) -> RunResult<()>;
}

/// For emulators, etc., which can have the function's return value retrieved as part of a function
/// call.
///
/// Implementations will iextract the values from the emulated machine's register file, stack frame
/// or whatever else, in a manner consistent with its calling convention.
#[allow(missing_docs)]
pub trait GetReturnValues {
    fn get_bool(&mut self) -> RunResult<bool>;
    fn get_i8(&mut self) -> RunResult<i8>;
    fn get_u8(&mut self) -> RunResult<u8>;
    fn get_i16(&mut self) -> RunResult<i16>;
    fn get_u16(&mut self) -> RunResult<u16>;
    fn get_u32(&mut self) -> RunResult<u32>;
    fn get_i32(&mut self) -> RunResult<i32>;
    fn get_f32(&mut self) -> RunResult<f32>;
}

/// Returns representative values for a given type
pub trait Parameters: std::cmp::PartialEq + Copy + std::fmt::Debug {
    /// Returns a few representative values
    fn vals() -> Vec<Self>
    where
        Self: std::marker::Sized;

    /// Returns a random value
    fn rand() -> Self;

    /// Pushes the parameter to the stack (or register file, or whereever) in preparation for a
    /// function call
    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()>;
}

/// Compares two values
pub trait ReturnValue:
    Copy + std::ops::BitXor<Output = Self> + Into<u128> + std::fmt::Debug
{
    /// Computes the Hamming distance between two objects of the same type
    fn distance(&self, other: &Self) -> f64 {
        let xor = *self ^ *other;
        let bits: u128 = xor.into();
        bits.count_ones() as f64
    }

    /// Pushes the parameter to the stack (or register file, or whereever) in preparation for a
    /// function call
    fn get<T: GetReturnValues>(r: &mut T) -> crate::RunResult<Self>;
}

impl ReturnValue for bool {
    fn get<T: GetReturnValues>(r: &mut T) -> crate::RunResult<bool> {
        r.get_bool()
    }
}

impl ReturnValue for u8 {
    fn get<T: GetReturnValues>(r: &mut T) -> crate::RunResult<u8> {
        r.get_u8()
    }
}

impl ReturnValue for u16 {
    fn get<T: GetReturnValues>(r: &mut T) -> crate::RunResult<u16> {
        r.get_u16()
    }
}

impl ReturnValue for u32 {
    fn get<T: GetReturnValues>(r: &mut T) -> crate::RunResult<u32> {
        r.get_u32()
    }
}

impl Parameters for bool {
    fn vals() -> Vec<Self> {
        vec![true, false]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_bool(*self)
    }
}

impl Parameters for u8 {
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

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_u8(*self)
    }
}

impl Parameters for i8 {
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

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_i8(*self)
    }
}

impl Parameters for i16 {
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
    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_i16(*self)
    }
}

impl Parameters for u16 {
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

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_u16(*self)
    }
}

impl Parameters for i32 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(i32::MAX - i);
            v.push(i32::MIN + i);
            v.push(-i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }
    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_i32(*self)
    }
}

impl Parameters for u32 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(u32::MAX - i);
            v.push(u32::MIN + i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_u32(*self)
    }
}

impl Parameters for f32 {
    fn vals() -> Vec<Self> {
        vec![0.0, -1.0, 1.0, -0.5, 0.5]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn put<T: TakeParameters>(&self, receiver: &mut T) -> crate::RunResult<()> {
        receiver.put_f32(*self)
    }
}

/// A trait for testing Callables.
///
/// An implementation might test a Callable against another one, or might read test cases from a
/// file or something
pub trait Tester<Input: Parameters, Output: ReturnValue> {
    /// Returns true if the callable passes the test, false otherwise
    fn tests_pass<T: Callable<Input, Output>>(&self, callable: &T) -> bool;
    /// Returns the total hamming distance between the callable's result and reference
    fn hamming_distance<T: Callable<Input, Output>>(&self, callable: &T) -> f64;
}

/// A fuzz tester.
///
/// Good for fuzz testing a Callable against another Callable (i.e., you want to generate a machine
/// code program that's equivalent to some function you have a pointer to)
#[derive(Debug)]
pub struct FuzzTest<Input: Parameters, Output: ReturnValue, Target: Callable<Input, Output>> {
    target: Target,
    test_cases: std::cell::RefCell<Vec<(Input, Output)>>,
}

impl<Input: Parameters, Output: ReturnValue, Target: Callable<Input, Output>>
    FuzzTest<Input, Output, Target>
{
    /// Constructs a new fuzz tester
    pub fn new(target: Target) -> Self {
        Self {
            target,
            test_cases: std::cell::RefCell::new(vec![]),
        }
    }

    /// Fuzz tests a callable against this test suite by calling each with random parameters. If any
    /// parameters are found to give differing return values for both callables, then this is a new
    /// failing test and it is added to the test suite.
    ///
    /// If such a test is found, then the method returns `false`, signifying that the test did not
    /// pass. Otherwise, the method returns `true`.
    pub fn fuzz<T: Callable<Input, Output>>(&self, candidate: &T) -> bool {
        for _ in 0..5000 {
            let i = Input::rand();
            if let Ok(r) = self.target.call(i) {
                if let Ok(s) = candidate.call(i) {
                    if (r.distance(&s)) != 0.0 {
                        // found a test case for which the putative function gives the wrong answer
                        self.test_cases.borrow_mut().push((i, r));
                        return false;
                    }
                } else {
                    // found a test case for which the putative function doesn't evaluate correctly
                    self.test_cases.borrow_mut().push((i, r));
                    return false;
                }
            }
        }
        true
    }
}

impl<Input: Parameters, Output: ReturnValue, Target: Callable<Input, Output>> Tester<Input, Output>
    for FuzzTest<Input, Output, Target>
{
    fn tests_pass<T: Callable<Input, Output>>(&self, callable: &T) -> bool {
        for t in self.test_cases.borrow().iter() {
            let Ok(r) = callable.call(t.0) else {
                // The function did not successfully return for the given inputs; the test failed.
                return false;
            };
            if r.distance(&t.1) != 0.0 {
                // The function doesn't pass the test because it returned some (valid) value
                // different from the expected one
                return false;
            }
        }
        self.fuzz(callable)
    }

    fn hamming_distance<T: Callable<Input, Output>>(&self, callable: &T) -> f64 {
        let mut result = 0.0f64;
        for t in self.test_cases.borrow().iter() {
            match callable.call(t.0) {
                Err(_) => {
                    // The function doesn't pass the test because of some error during execution
                    return f64::MAX;
                }
                Ok(r) => {
                    result += r.distance(&t.1);
                }
            }
        }
        result
    }
}
