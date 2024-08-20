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

pub mod sequence;

pub trait Iterable {
    //! A trait for anything that can be iterated across in an exhaustive manner. For example, the
    //! Bruteforce search uses this.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step. Returns true if the end of the iteration has not been reached.
    fn step(&mut self) -> bool;
}

pub trait PrunedSearch<P> {
    //! A trait for performing a brute-force search that has some instructions pruned away. The type
    //! P represents the prune.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step. Returns true if the end of the iteration has not been reached.
    fn pruned_step(&mut self, prune: &P) -> bool;
}

pub trait Random {
    //! A trait for things that can be searched through randomly. For example, the stochastic
    //! search uses this.

    /// Start from a random point
    fn random() -> Self;

    /// Take a step in a random direction
    fn step(&mut self);
}

pub trait Goto<I> {
    //! Trait for starting a search from a particular point in the search space.

    /// Replace self with some other value
    fn goto(&mut self, destination: &[I]);
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
/// been violated. If the constraints are satisfied and not violated, then this case is represented
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

/// To get the search space down to something manageable, a type could implement this trait to
/// reduce the number of instructions considered. This might be used to make sure the instructions
/// only reads from the permitted registers or memory locations for example, or might write-protect
/// regions of memory, or remove from consideration instructions incompatible with this or that CPU
/// variant or whatever.
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
