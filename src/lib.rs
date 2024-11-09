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

/// A type implementing the Constraint trait can constrain the search space to, for example,
/// leaf functions, or programs compatible with certain variants, or programs not liable to be
/// modified by peephole optimization, etc. etc.
pub trait Constrain<T> {
    /// Fixes the candidate up in a deterministic way, compatible with the `BruteForce` search.
    fn fixup(&mut self);

    /// Fixes the candidate up in a stochastic way
    fn stochastic_fixup(&mut self) {
        self.fixup();
    }

    /// Reports on whether this constraint would make any changes to the program
    fn report(&self, offset: usize) -> Vec<String>;
}

/// Disassembles the code sequence, and prints out the lints along the way {
pub fn report<I: std::fmt::Display, C: Constrain<I>>(sequence: &Sequence<I>, constraint: &C) {
    for offset in 0..sequence.len() {
        for report in constraint.report(offset) {
            println!("\t; {report}");
        }
        println!("\t{}", sequence[offset]);
    }
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

pub trait DataFlow<T> {
    //! A trait for very local dataflow. It's generic across `T`, a type intended to represent
    //! "things" a machine instruction may read from or write to.
    //!
    //! For example, a type representing a Z80 machine code instruction could implement this for
    //! the Z80's register file, the flags, the I/O space and the address space.

    /// returns true iff the variable `t` is read (used) by the instruction or basic block before
    /// any assignment. Such a variables must be live at the start of the block.
    fn reads(&self, t: &T) -> bool;

    /// returns true iff the variable `t` is assigned (written to) by the instruction or basic
    /// block, effectively "killing" any previous value it held.
    fn writes(&self, t: &T) -> bool;

    /// Modifies the instruction so that it reads from `t`.
    fn make_read(&mut self, t: &T) -> bool;

    /// Modifies the instruction so that it writes to `t`.
    fn make_write(&mut self, t: &T) -> bool;
}
