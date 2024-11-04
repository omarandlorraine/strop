use crate::test;
use crate::test::Vals;
use crate::Callable;
use crate::Iterable;
use crate::StropError;

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
    pub fn new(target_function: T, initial_candidate: U) -> Self {
        let candidate = initial_candidate;
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
            self.candidate.fixup();

            match test::passes(&self.candidate, &self.tests) {
                Err(StropError::DidntReturn) => {
                    // The candidate does not pass the test case(s)
                    // go round the loop again
                }
                Err(StropError::Undefined) => {
                    // The candidate does not pass the test case(s)
                    // go round the loop again
                }
                Ok(false) => {
                    // The candidate does not pass the test case(s)
                    // go round the loop again
                }
                Ok(true) => {
                    // Found a candidate which passes all known test cases.
                    // Let's fuzz test the candidate
                    if let Some(test_case) =
                        test::fuzz(&self.target_function, &self.candidate, 5000)
                    {
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
}
