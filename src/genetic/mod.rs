mod generate;
use crate::test::{Input, Output};
pub use generate::Generate;

use crate::Callable;
use crate::TestSuite;

#[derive(Clone, Debug)]
struct ScoredCandidate<InputParameters, ReturnValue, T: Callable<InputParameters, ReturnValue>> {
    score: f64,
    candidate: T,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
}

impl<InputParameters: Input, ReturnValue: Output, U: Callable<InputParameters, ReturnValue>>
    AsRef<U> for ScoredCandidate<InputParameters, ReturnValue, U>
{
    fn as_ref(&self) -> &U {
        &self.candidate
    }
}

impl<InputParameters: Input, ReturnValue: Output, U: Callable<InputParameters, ReturnValue>>
    ScoredCandidate<InputParameters, ReturnValue, U>
{
    pub fn new(candidate: U, tests: &TestSuite<InputParameters, ReturnValue>) -> Self {
        let mut s = Self {
            score: 0.0,
            candidate,
            input: Default::default(),
            ret: Default::default(),
        };
        s.retest(tests);
        s
    }

    fn retest(&mut self, tests: &TestSuite<InputParameters, ReturnValue>) {
        self.score = tests.score(&self.candidate);
    }
}
