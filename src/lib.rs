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

mod sequence;
pub use sequence::Sequence;

pub mod test;

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

    /// Take one stride (this skips many points in the search space which `step` would have
    /// visited; so that many known bad points are skipped.) Returns true if the end of the
    /// iteration has not been reached.
    fn stride(&mut self) -> bool;
}

pub trait IterableSequence {
    //! A trait for anything that can be iterated across in an exhaustive manner, but which also
    //! may be stepped at a particular offset.
    //!
    //! For example, if a sequence of instructions is found by static analysis to have an incorrect
    //! instruction at offset *x*, then the sequence's `step_at` method may be called, informing
    //! the search strategy of this fact.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step. Returns true if the end of the iteration has not been reached.
    fn step_at(&mut self, offset: usize) -> bool;

    /// Take one stride. Returns true if the end of the iteration has not been reached.
    fn stride_at(&mut self, offset: usize) -> bool;
}

impl<T: IterableSequence> Iterable for T {
    fn first() -> Self {
        <Self as IterableSequence>::first()
    }
    fn step(&mut self) -> bool {
        self.step_at(0)
    }
    fn stride(&mut self) -> bool {
        self.stride_at(0)
    }
}

pub trait PrunedSearch<Prune> {
    //! A trait for performing a brute-force search that has some instructions pruned away. The type
    //! Prune represents the prune.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step. Returns true if the end of the iteration has not been reached.
    fn pruned_step(&mut self, prune: &Prune) -> bool;
}

pub trait Random {
    //! A trait for things that can be searched through randomly. For example, the stochastic
    //! search uses this.

    /// Start from a random point
    fn random() -> Self;

    /// Take a step in a random direction
    fn step(&mut self);
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

/// Type `ConstraintViolation` represents the possibility that an unary or binary constraint has
/// been violated.
///
/// If the constraints are satisfied and not violated, then this case is represented
/// by the variant `Ok`. If a constraint has been violated, then if a suitable replacement would be
/// found by successive calls to the `Iterable` trait's `step` method, then it is held in the
/// `ReplaceWith` variant. If a constraint has been violated but no such replacement can be found,
/// then this case is represented by the `Violation` variant.
#[derive(Debug)]
pub enum ConstraintViolation<T> {
    /// The proposed value was not found to violate any unary constraints
    Ok,

    /// The proposed value violated a constraint, and here is a proposed replacement. The proposed
    /// replacement would also be found by successive calls to the `Iterable` trait's `step`
    /// method.
    ReplaceWith(T),

    /// The proposed value violated a constraint, but we could not find a suitable replacement.
    Violation,
}

/// A type could implement this trait to reduce the number of instructions considered.
///
/// This might be used to make sure the instructions only reads from the permitted registers or
/// memory locations for example, or might write-protect regions of memory, or remove from
/// consideration instructions incompatible with this or that CPU variant or whatever.
pub trait Prune<T> {
    /// Considers the `T` passed to this method, and if the instruction is to be pruned away from
    /// the search, returns a `ConstraintViolation<T>` that describes how to proceed with the
    /// search.
    fn prune(&self, t: &T) -> ConstraintViolation<T>;
}

pub trait ConstraintSatisfaction<T> {
    //! A trait for constraint solvers
    /// Considers the `T` passed to this method, and checks if it violates any unary constraints.
    fn unary(&self, t: &T) -> ConstraintViolation<T>;

    /// Considers the two connected nodes of type `T`, and sees if they violate any binary
    /// constraints.
    fn binary(&self, a: &T, b: &T) -> ConstraintViolation<T>;
}

pub trait CallingConvention<SamplePoint, InputParameters, ReturnValue> {
    //! A trait for calling conventions. A type which implements this trait can execute a function
    //! taking the given argument(s), and return the function's return value.

    /// Calls the given callable object, passing it the parameters of type `InputParameters`, and returning an
    /// `ReturnValue`.
    fn call(function: &SamplePoint, parameters: InputParameters)
        -> Result<ReturnValue, StropError>;
}

/// Enumerates reasons why executing a function may fail
#[derive(Debug, PartialEq)]
pub enum StropError {
    /// The callable object does not pass static analysis, but the static analysis pass has
    /// proposed for a step to be taken at the given offset
    Step(usize),

    /// The callable object does not pass static analysis, but the static analysis pass has
    /// proposed for a stride to be taken at the given offset
    Stride(usize),

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

    /// Performs a static analysis on the object
    fn fixup(&mut self) {}

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
