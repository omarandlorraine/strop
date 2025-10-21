//! Strop, the stochastic optimizer written in Rust
//! -----------------------------------------------
//! This program generates assembly language programs that compute a given function.
//!
//! Strop provides mechanisms for generating programs, and mutating them in ways that
//! stochastically approach the desired output.
//!
//! Strop also provides a bruteforce search.
//!
//! Another way to describe strop, is that it generates pretty good assembly programs, either
//! randomly or by bruteforce search.
//!
//! So far, strop has had a focus on supporting architectures that are not well supported by
//! mainstream compilers such as LLVM. These have included architectures common in low-end
//! embedded, and hobbyist retrocomputing.

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
#![forbid(unsafe_code)]

pub mod backends;
mod sequence;
pub mod static_analysis;
pub use static_analysis::{Fixup, StaticAnalysis};
pub mod dataflow;
pub mod test;
mod triplets;
pub use triplets::Triplet;
pub mod search;

pub use sequence::Sequence;

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

    /// The function takes too many arguments for the calling convention to handle
    TooManyArguments,

    /// The function has too many return values for the calling convention to handle
    TooManyReturnValues,

    /// The function signature contains types not supported by the calling convention
    UnsupportedType,
}

/// Return type for in-place iteration
pub type RunResult<T> = Result<T, RunError>;

/// Trait representing an instruction
pub trait Instruction: std::fmt::Display + std::fmt::Debug {
    /// Return a random instruction
    fn random() -> Self;

    /// Applies a random mutation to an instruction
    fn mutate(&mut self);

    /// Returns the first instruction
    fn first() -> Self;

    /// Increments the instruction
    fn increment(&mut self) -> IterationResult;

    /// returns the machine-code representation in bytes;
    fn to_bytes(&self) -> Vec<u8>;

    /// disassembles a sequence of bytes into one machine instruction
    fn from_bytes(bytes: &[u8]) -> Self;
}

pub trait Callable<Input, Output> {
    //! A trait for objects which may be called.
    //!
    //! Implementations may represent native functions, Lisp expressions, machine code programs
    //! executed in an emulator, function pointers, etc.

    /// Calls the given callable object
    fn call(&self, parameters: Input) -> RunResult<Output>;
}

impl<Input, Output> Callable<Input, Output> for fn(Input) -> RunResult<Output> {
    fn call(&self, parameters: Input) -> RunResult<Output> {
        (self)(parameters)
    }
}

pub trait Traverse {
    //! A trait for functions that may be searched for.
    //!
    //! Implementations typically represent machine code for such-and-such an architecture
    //! complying with this or that calling convention

    /// Steps through the search space
    fn increment(&mut self);

    /// Applies a random mutation to the putative program
    fn mutate(&mut self);

    /// Constructs such an object from a sequence of bytes
    fn from_bytes(bytes: &[u8]) -> Self;
}

pub trait Testable: std::fmt::Display + std::fmt::Debug {
    //! A trait for functions that may be searched for.
    //!
    //! Implementations typically represent machine code for such-and-such an architecture
    //! complying with this or that calling convention

    /// Steps through the search space
    fn increment(&mut self);

    /// Steps through the search space until the test cases pass
    fn next(&mut self);

    /// Returns true if all the test cases pass, false otherwise
    fn pass(&self) -> bool;

    /// Applies a random mutation to the putative program
    fn mutate(&mut self);
}

#[cfg(test)]
pub mod generic_unit_tests;
