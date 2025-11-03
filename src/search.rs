//! This module

use crate::Callable;

/// Searcher exists to join a tester, such as a fuzz tester, and a search space, such as an assembly-language program.
pub struct Searcher<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display,
> {
    test: T,
    s: S,
    _phantom_marker: std::marker::PhantomData<(Input, Output)>,
}

impl<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display + std::fmt::Debug,
> crate::Callable<Input, Output> for Searcher<Input, Output, T, S>
{
    fn call(&self, i: Input) -> crate::RunResult<Output> {
        self.s.call(i)
    }
}

impl<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display + std::fmt::Debug,
> std::fmt::Debug for Searcher<Input, Output, T, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.s)
    }
}

impl<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display,
> std::fmt::Display for Searcher<Input, Output, T, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.s)
    }
}

impl<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display + std::fmt::Debug,
> crate::Testable for Searcher<Input, Output, T, S>
{
    fn increment(&mut self) {
        self.s.increment();
    }
    fn next(&mut self) {
        self.s.increment();
        while !self.pass() {
            self.s.increment();
        }
    }
    fn mutate(&mut self) {
        self.s.mutate();
    }
    fn pass(&self) -> bool {
        self.test.tests_pass(&self.s)
    }
}

impl<
    Input: crate::test::Parameters,
    Output: crate::test::ReturnValue,
    T: crate::test::Tester<Input, Output>,
    S: crate::Traverse + Callable<Input, Output> + std::fmt::Display,
> Searcher<Input, Output, T, S>
{
    /// Constructs a new Searcher.
    pub fn new(s: S, test: T) -> Self {
        Self {
            s,
            test,
            _phantom_marker: Default::default(),
        }
    }
}
