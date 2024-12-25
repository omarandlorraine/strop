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
#[cfg(feature = "m68k")]
pub mod m68000;
#[cfg(feature = "m6809")]
pub mod m6809;
#[cfg(feature = "z80")]
pub mod z80;

pub mod dataflow;
pub mod peephole;

mod sequence;
pub use sequence::Sequence;

pub mod test;

mod genetic;
pub use genetic::Generate;

mod bruteforce;
pub use bruteforce::BruteForce;

pub trait Disassemble {
    //! A trait for printing out the disassembly of an instruction, a subroutine, or anything else

    /// Disassemble to stdout
    fn dasm(&self);
}

pub trait Iterable {
    //! A trait for anything that can be iterated across in an exhaustive manner. For example, the
    //! Bruteforce search uses this.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step. Returns true if the end of the iteration has not been reached.
    fn step(&mut self) -> bool;
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

pub trait Goto<SamplePoint> {
    //! Trait for starting a search from a particular point in the search space.

    /// Replace self with some other value
    fn goto(&mut self, destination: &[SamplePoint]);
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

/// A type implementing the Constraint trait can constrain the search space to, for example,
/// leaf functions, or programs compatible with certain variants, or programs not liable to be
/// modified by peephole optimization, etc. etc.
pub trait Constrain<Insn> {
    /// Fixes the candidate up in a deterministic way, compatible with the `BruteForce` search.
    fn fixup(&self, candidate: &mut Sequence<Insn>) -> Option<(usize, &'static str)>;

    /// Fixes the candidate up in a stochastic way
    fn stochastic_fixup(&self, candidate: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        self.fixup(candidate)
    }
}

/// Enumerates reasons why executing a function may fail
#[derive(Debug, PartialEq)]
pub enum StropError {
    /// The represented function is not defined for the given inputs
    Undefined,

    /// The callable object ran amok during emulation, or somehow did not return
    DidntReturn,
}

pub trait Callable<InputParameters, ReturnValue> {
    //! A trait for objects which may be called.
    //!
    //! For example, these could be machine code programs associated with a particular calling
    //! convention ready for execution in an emulated environment, or they may be function
    //! pointers, or lisp expressions, etc.)

    /// Calls the given callable object
    fn call(&self, parameters: InputParameters) -> Result<ReturnValue, StropError>;
}

impl<InputParameters, ReturnValue> Callable<InputParameters, ReturnValue>
    for fn(InputParameters) -> Result<ReturnValue, StropError>
{
    fn call(&self, parameters: InputParameters) -> Result<ReturnValue, StropError> {
        (self)(parameters)
    }
}
