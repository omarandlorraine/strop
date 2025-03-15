use crate::test;
use crate::test::Vals;
use crate::Callable;
use crate::Step;

/// Performs a brute force search over a given search space `Searchable`
#[derive(Debug, Clone)]
pub struct BruteForce<
    InputParameters,
    ReturnValue: Clone,
    TargetFunction: Callable<InputParameters, ReturnValue>,
    Searchable: Callable<InputParameters, ReturnValue> + Step,
> {
    target_function: TargetFunction,
    candidate: Searchable,
    tests: Vec<(InputParameters, ReturnValue)>,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,

    /// Keeps track of how many iterations the bruteforce search has been through.
    pub count: usize,
}

impl<
        InputParameters: Copy + Vals,
        ReturnValue: Vals + std::cmp::PartialEq + Clone,
        TargetFunction: Callable<InputParameters, ReturnValue>,
        Searchable: Callable<InputParameters, ReturnValue> + Step + Clone,
    > BruteForce<InputParameters, ReturnValue, TargetFunction, Searchable>
{
    /// Constructs a new `BruteForce`
    pub fn new(target_function: TargetFunction, initial_candidate: Searchable) -> Self {
        let candidate = initial_candidate;
        let tests = test::quick_tests(&target_function);
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
    pub fn step(&mut self) -> crate::IterationResult {
        self.count += 1;
        self.candidate.next()?;
        self.candidate.dataflow_fixup();
        Ok(())
    }

    /// Tests that the candidate matches the target function
    pub fn test(&mut self) -> bool {
        match test::passes(&self.candidate, &self.tests) {
            Err(_) => {
                // The candidate does not pass the test case(s)
                println!("Err");
                false
            }
            Ok(false) => {
                // The candidate does not pass the test case(s)
                println!("Ok(false)");
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
    pub fn search(&mut self) -> Option<Searchable> {
        loop {
            self.step().ok()?;
            if self.test() {
                return Some(self.candidate.clone());
            }
        }
    }
}
