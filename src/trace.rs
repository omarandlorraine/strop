use crate::Callable;
use crate::Disassemble;
use crate::IterationResult;
use crate::RunResult;
use crate::Step;

/// A wrapper which writes to stdout
#[derive(Debug, Clone)]
pub struct Trace<T: Disassemble + Clone>(T);

/// Wraps an object up in a `Trace`, so that each mutation is printed to stdout.
pub trait ToTrace {
    /// Wraps an object up in a `Trace`, so that each mutation is printed to stdout.
    fn trace(self) -> Trace<Self>
    where
        Self: Sized + Clone + Disassemble,
    {
        Trace::<Self>(self)
    }
}

impl<T> ToTrace for T {}

impl<T: Disassemble + Clone> Disassemble for Trace<T> {
    fn dasm(&self) {
        self.0.dasm();
    }
}

impl<Insn, S: crate::BruteforceSearch<Insn> + Clone + Disassemble> crate::BruteforceSearch<Insn>
    for Trace<S>
{
    fn analyze_this(&self) -> Option<crate::StaticAnalysis<Insn>> {
        None
    }
    fn inner(&mut self) -> &mut dyn crate::BruteforceSearch<Insn> {
        &mut self.0
    }
    fn step(&mut self) {
        self.inner().step();
        println!("stepped:");
        self.dasm();
    }
}

impl<T: Step + Clone + Disassemble> Step for Trace<T> {
    fn next(&mut self) -> IterationResult {
        self.0.next()?;
        println!("trace:");
        self.0.dasm();
        Ok(())
    }

    fn first() -> Self {
        Self(T::first())
    }
}

impl<
        InputParameters,
        ReturnType,
        T: Callable<InputParameters, ReturnType> + Clone + Disassemble,
    > Callable<InputParameters, ReturnType> for Trace<T>
{
    fn call(&self, parameters: InputParameters) -> RunResult<ReturnType> {
        self.0.call(parameters)
    }
}
