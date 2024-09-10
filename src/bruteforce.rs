use crate::test;
use crate::test::Vals;
use crate::Callable;
use crate::Iterable;

/// Performs a brute force search over a given search space `U`
#[derive(Debug)]
pub struct BruteForce<
    InputParameters,
    ReturnValue,
    T: Callable<InputParameters, ReturnValue>,
    U: Callable<InputParameters, ReturnValue> + Iterable,
> {
    target_function: T,
    candidate: U,
    tests: Vec<(InputParameters, ReturnValue)>,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
}

impl<
        InputParameters: Copy + Vals,
        ReturnValue: Vals + std::cmp::PartialEq,
        T: Callable<InputParameters, ReturnValue>,
        U: Callable<InputParameters, ReturnValue> + Iterable + Clone,
    > BruteForce<InputParameters, ReturnValue, T, U>
{
    /// Constructs a new `BruteForce`
    pub fn new(target_function: T) -> Self {
        let candidate = U::first();
        let tests = test::quick_tests(&target_function);
        Self {
            target_function,
            candidate,
            tests,
            input: std::marker::PhantomData,
            ret: std::marker::PhantomData,
        }
    }

    /// Returns the next function that matches the target function
    pub fn search(&mut self) -> Option<U> {
        loop {
            if !self.candidate.step() {
                return None;
            }

            if test::passes(&self.candidate, &self.tests) {
                // Found a candidate which passes all known test cases.
                // Let's fuzz test the candidate
                if let Some(test_case) = test::fuzz(&self.target_function, &self.candidate, 5000) {
                    // We've fuzzed the functions against eachother and found another test case.
                    // So keep hold of this new test case
                    self.tests.push(test_case);
                } else {
                    // The candidate passed all known test cases and also a fuzz test, so let's say
                    // it's good enough and return it
                    return Some(self.candidate.clone());
                }
            }
        }
    }
}
