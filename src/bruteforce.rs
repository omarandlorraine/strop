//! Implements a bruteforce search

use crate::Callable;
use crate::Disassemble;
use crate::TestSuite;
use crate::test::Vals;

/// Trait for things that can be bruteforce searched
pub trait BruteForceSearch {
    /// Steps to the next value in the search space, and then applies static analysis fixups.
    fn next(&mut self) -> crate::IterationResult;
}

/// Performs a brute force search over a given search space `Searchable`
#[derive(Debug, Clone)]
pub struct BruteForce<
    InputParameters,
    ReturnValue: Clone,
    TargetFunction: Callable<InputParameters, ReturnValue>,
    Searchable: Callable<InputParameters, ReturnValue> + BruteForceSearch + Disassemble,
> {
    target_function: TargetFunction,
    candidate: Searchable,
    tests: TestSuite<InputParameters, ReturnValue>,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,

    /// Keeps track of how many iterations the bruteforce search has been through.
    pub count: usize,
}

impl<
    InputParameters,
    ReturnValue: Clone,
    TargetFunction: Callable<InputParameters, ReturnValue>,
    Searchable: Callable<InputParameters, ReturnValue> + BruteForceSearch + Disassemble,
> crate::Disassemble for BruteForce<
    InputParameters,
    ReturnValue,
    TargetFunction,
    Searchable,
> {
    fn dasm(&self) {
        self.candidate.dasm()
    }
}


impl<
    InputParameters: Copy + Vals,
    ReturnValue: Vals + std::cmp::PartialEq + Clone,
    TargetFunction: Callable<InputParameters, ReturnValue>,
    Searchable: Callable<InputParameters, ReturnValue> + BruteForceSearch + Clone + Disassemble,
> BruteForce<InputParameters, ReturnValue, TargetFunction, Searchable>
{
    /// Constructs a new `BruteForce`
    pub fn new(target_function: TargetFunction, initial_candidate: Searchable) -> Self {
        let candidate = initial_candidate;
        let tests = TestSuite::generate(&target_function);
        Self {
            target_function,
            candidate,
            tests,
            input: std::marker::PhantomData,
            ret: std::marker::PhantomData,
            count: 0,
        }
    }

    /// Returns the candidate currently under consideration
    pub fn candidate(&self) -> &Searchable {
        &self.candidate
    }

    /// Advances the candidate to the next position in the search space
    pub fn next(&mut self) -> crate::IterationResult {
        self.count += 1;
        self.candidate.next()?;
        Ok(())
    }

    /// Tests that the candidate matches the target function
    pub fn test(&mut self) -> bool {
        self.tests
            .checked_fuzz(&self.target_function, &self.candidate, 5000)
    }

    /// Returns the next function that matches the target function
    pub fn search(&mut self) -> Option<Searchable> {
        loop {
            self.next().ok()?;
            if self.test() {
                return Some(self.candidate.clone());
            }
        }
    }
}
