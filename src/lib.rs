//! Superoptimizer written in Rust
//! ------------------------------
//! This program stochastically generates assembly language programs that compute a given function.
//! Strop provides mechanisms for generating programs, and mutating them in ways that
//! stochastically approach the desired output.
//!
//! Another way to describe strop, is that it randomly generates pretty good assembly programs.
//!
//! So far, strop has had a focus on supporting architectures that are not well supported by
//! mainstream compilers such as LLVM. These have included architectures common in low-end
//! embedded, and hobbyist retrocomputing.
//!

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
#![forbid(unsafe_code)]

#[cfg(feature = "armv4t")]
pub mod armv4t;
#[cfg(feature = "m6502")]
pub mod m6502;
#[cfg(feature = "m6809")]
pub mod m6809;
#[cfg(feature = "mips")]
pub mod mips;
#[cfg(feature = "z80")]
pub mod z80;

mod sequence;
pub use sequence::Sequence;

pub mod test;

mod genetic;
pub use genetic::Generate;

mod bruteforce;
pub use bruteforce::{BruteForce, ToBruteForce};

mod subroutine;
pub use subroutine::ShouldReturn;

mod trace;
pub use trace::{ToTrace, Trace};

pub mod branches;
pub mod dataflow;
pub use branches::Branch;

/// Result of a static analysis pass. Explains why a code sequence has been found to be illogical
/// or unsuitable, and provides a way to prune such a sequence from the search.
pub struct StaticAnalysis<Instruction> {
    /// Specifies at what offset into this sequence the problem was found
    pub offset: usize,
    advance: fn(&mut Instruction) -> IterationResult,
    /// Human-readable description of the problem
    pub reason: &'static str,
}

impl<Instruction> StaticAnalysis<Instruction> {
    /// Constructs an Err(self)
    pub fn err(
        reason: &'static str,
        advance: fn(&mut Instruction) -> IterationResult,
        offset: usize,
    ) -> Result<(), Self> {
        Err(Self {
            offset,
            advance,
            reason,
        })
    }

    /// Constructs an Ok(())
    pub fn ok() -> Result<(), Self> {
        Ok(())
    }
}

impl<Instruction> StaticAnalysis<Instruction> {
    /// Constructs a new `StaticAnalysis` object, replacing the offset value.
    pub fn set_offset(&self, offset: usize) -> Self {
        let Self {
            offset: _,
            advance,
            reason,
        } = self;
        Self {
            offset,
            advance: *advance,
            reason,
        }
    }
}

impl<Instruction> std::fmt::Debug for StaticAnalysis<Instruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "StaticAnalysis {} offset {}", self.reason, self.offset)
    }
}

/// Impl this on a datatype that may be iterated by mutating the datum in place. This is then used
/// by the library to perform bruteforce searches and such
pub trait Step {
    /// Advances the value to the next state.
    /// Returns `Ok(())` if the step was successful.
    /// Returns `Err(StepError::End)` if the end has been reached.
    fn next(&mut self) -> IterationResult;

    /// Returns the first value
    fn first() -> Self;
}

/// Impl this trait on any code sequence (a subroutine, a function, other passes) so that the brute
/// force search can mutate and query it.
pub trait BruteforceSearch<Insn> {
    /// Optionally return a `StaticAnalysis` if a code sequence is found to be problematic or in some
    /// way suboptimal.
    fn analyze_this(&self) -> Result<(), StaticAnalysis<Insn>>;

    /// Returns either this pass's `StaticAnalysis<Insn>` or the inner's
    fn analyze(&mut self) -> Result<(), StaticAnalysis<Insn>> {
        self.inner().analyze()?;
        self.analyze_this()
    }

    /// Since client code can arbitrarily chain these passes together, return the next node in the
    /// "linked list".
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn>;

    /// Step through the search space. Apply any static analysis results.
    fn step(&mut self) {
        self.inner().step();
        self.fixup();
    }

    /// Applies a `StaticAnalysis`, which means fixing whatever problem the `StaticAnalysis`
    /// represents.
    fn apply(&mut self, static_analysis: &StaticAnalysis<Insn>) {
        self.inner().apply(static_analysis);
    }

    /// Applies all `StaticAnalysis` instances.
    fn fixup(&mut self) {
        while let Err(sa) = self.analyze() {
            self.apply(&sa);
        }
    }
}

/// Enum representing possible errors when stepping
#[derive(Debug, PartialEq, Eq)]
pub enum StepError {
    /// No more possible values.
    End,
}

/// Return type for in-place iteration
pub type IterationResult = Result<(), StepError>;

/// Enum representing possible errors when running (a subroutine, a function, an interrupt handler,
/// etc...)
#[derive(Debug, PartialEq, Eq)]
pub enum RunError {
    /// The program ran amok (it jumped to some location outside of itself, or caused a runtime
    /// exception, or undefined behavior, or ...)
    RanAmok,

    /// The function is not defined for the given parameters
    NotDefined,
}

/// Return type for in-place iteration
pub type RunResult<T> = Result<T, RunError>;

/// Trait for returning a BruteForce object
pub trait AsBruteforce<
    Insn,
    InputParameters,
    ReturnType: Clone,
    Function: Callable<InputParameters, ReturnType>,
>: Callable<InputParameters, ReturnType> + Clone + BruteforceSearch<Insn>
{
    /// Returns a `BruteForce`
    fn bruteforce(
        self,
        function: Function,
    ) -> BruteForce<Insn, InputParameters, ReturnType, Function, Self>;
}

pub trait Disassemble {
    //! A trait for printing out the disassembly of an instruction, a subroutine, or anything else

    /// Disassemble to stdout
    fn dasm(&self);
}

pub trait Mutate {
    //! A trait for anything that can be randomly mutated

    /// Returns a random value
    fn random() -> Self;

    /// Mutates the object in some random way
    fn mutate(&mut self);
}

pub trait Crossover {
    //! A trait for taking two items having the same type, and producing a thrid item of the same
    //! type, having a value being a mashup of the two parents. Such a thing is used in the genetic
    //! algorithm

    /// spawns a child from two parents
    fn crossover(a: &Self, b: &Self) -> Self;
}

pub trait Goto<Insn> {
    //! Trait for starting a search from a particular point in the search space.

    /// Replace self with some other value
    fn goto(&mut self, destination: &[Insn]);
}

impl<Insn: Clone, S: Clone + AsMut<Sequence<Insn>>> Goto<Insn> for S {
    fn goto(&mut self, destination: &[Insn]) {
        let s = self.as_mut();
        s.goto(destination);
    }
}

pub trait Encode<T> {
    //! Trait for things that can be converted to sequences (of bytes, words, etc)

    /// Return the length of the encoding
    fn len(&self) -> usize {
        self.encode().len()
    }

    /// Returns `true` if `encode()` would return an empty vector, false otherwise
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the encoding
    fn encode(&self) -> Vec<T>;
}

pub trait Callable<InputParameters, ReturnValue> {
    //! A trait for objects which may be called.
    //!
    //! For example, these could be machine code programs associated with a particular calling
    //! convention ready for execution in an emulated environment, or they may be function
    //! pointers, or lisp expressions, etc.)

    /// Calls the given callable object
    fn call(&self, parameters: InputParameters) -> RunResult<ReturnValue>;
}

impl<InputParameters, ReturnValue> Callable<InputParameters, ReturnValue>
    for fn(InputParameters) -> RunResult<ReturnValue>
{
    fn call(&self, parameters: InputParameters) -> RunResult<ReturnValue> {
        (self)(parameters)
    }
}

/// Objective function. The genetic algorithms try to minimize this function. Possible functions
/// include "length of program" (the algorithm tries to reduce this, so it will find the shortest
/// program), and "average runtime in machine cycles" (the algorithm tries to reduce this, so it
/// will find faster programs).
pub trait Objective<Something> {
    /// Evaluates the objective function
    fn score(&self, something: &Something) -> f64;
}
