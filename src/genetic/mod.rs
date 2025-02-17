mod generate;
pub use generate::Generate;

use crate::test::Vals;
use crate::Callable;

#[derive(Clone, Debug)]
struct ScoredCandidate<InputParameters, ReturnValue, T: Callable<InputParameters, ReturnValue>> {
    score: f64,
    candidate: T,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
}

impl<InputParameters: Vals, ReturnValue: Vals, U: Callable<InputParameters, ReturnValue>> AsRef<U>
    for ScoredCandidate<InputParameters, ReturnValue, U>
{
    fn as_ref(&self) -> &U {
        &self.candidate
    }
}

impl<InputParameters: Vals, ReturnValue: Vals, U: Callable<InputParameters, ReturnValue>>
    ScoredCandidate<InputParameters, ReturnValue, U>
{
    pub fn new(candidate: U, tests: &Vec<(InputParameters, ReturnValue)>) -> Self {
        let mut s = Self {
            score: 0.0,
            candidate,
            input: Default::default(),
            ret: Default::default(),
        };
        s.retest(tests);
        s
    }

    fn retest(&mut self, tests: &Vec<(InputParameters, ReturnValue)>) {
        self.score = crate::test::score(&self.candidate, tests);
    }
}
