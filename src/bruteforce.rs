use crate::test;
use crate::test::Vals;
use crate::Callable;
use crate::Constrain;
use crate::Iterable;
use crate::StropError;

/// Performs a brute force search over a given search space `U`
#[derive(Debug, Clone)]
pub struct BruteForce<
    InputParameters,
    ReturnValue: Clone,
    T: Callable<InputParameters, ReturnValue> + Clone,
    U: Callable<InputParameters, ReturnValue> + Iterable,
    Insn: Clone,
> {
    target_function: T,
    candidate: U,
    tests: Vec<(InputParameters, ReturnValue)>,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
    insn: std::marker::PhantomData<Insn>,
    trace_enable: bool,
    count: usize,
}

impl<
        Insn: Clone,
        InputParameters: Copy + Vals,
        ReturnValue: Vals + std::cmp::PartialEq + Clone,
        T: Callable<InputParameters, ReturnValue> + Clone,
        U: Callable<InputParameters, ReturnValue>
            + Iterable
            + Clone
            + crate::Disassemble
            + Constrain<Insn>,
    > BruteForce<InputParameters, ReturnValue, T, U, Insn>
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
            insn: std::marker::PhantomData,
            trace_enable: false,
            count: 0,
        }
    }

    /// Enables trace: disassembles each candidate to stdout.
    pub fn trace(&mut self) -> Self {
        self.trace_enable = true;
        self.clone()
    }

    /// Returns the candidate currently under consideration
    pub fn candidate(&self) -> &U {
        &self.candidate
    }

    /// Prints the current candidate to stdout
    pub fn dasm(&self) {
        println!("\ncandidate{}:", self.count);
        self.candidate().dasm();
    }

    /// Advances the candidate to the next position in the search space
    pub fn step(&mut self) -> bool {
        self.count += 1;
        if !self.candidate.step() {
            if self.trace_enable {
                self.dasm();
            }
            return false;
        }
        self.candidate.fixup();
        if self.trace_enable {
            self.dasm();
        }
        true
    }

    /// Tests that the candidate matches the target function
    pub fn test(&mut self) -> bool {
        match test::passes(&self.candidate, &self.tests) {
            Err(StropError::DidntReturn) => {
                // The candidate does not pass the test case(s)
                false
            }
            Err(StropError::Undefined) => {
                // The candidate does not pass the test case(s)
                false
            }
            Ok(false) => {
                // The candidate does not pass the test case(s)
                false
            }
            Ok(true) => {
                // Found a candidate which passes all known test cases.
                // Let's fuzz test the candidate
                if let Some(test_case) = test::fuzz(&self.target_function, &self.candidate, 5000) {
                    // We've fuzzed the functions against eachother and found another test case.
                    // So keep hold of this new test case
                    self.tests.push(test_case);
                    false
                } else {
                    // The candidate passed all known test cases and also a fuzz test, so let's say
                    // it's good enough and return it
                    true
                }
            }
        }
    }

    /// Returns the next function that matches the target function
    pub fn search(&mut self) -> Option<U> {
        loop {
            self.step();
            if self.test() {
                return Some(self.candidate.clone());
            }
        }
    }
}
