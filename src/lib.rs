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

pub mod sequence;

pub trait Iterable {
    //! A trait for anything that can be iterated across in an exhaustive manner. For example, the
    //! Bruteforce search uses this.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step
    fn step(&mut self) -> bool;

    /// Replace self with some other value
    fn goto(&mut self, destination: &Self);
}

pub trait ConstraintSatisfactionSolver {
    //! A trait for constraint satisfaction solvers. These are like exhaustive searches, but
    //! consider relationships between consecutive instructions to reduce the search space.

    /// Start from the beginning
    fn first() -> Self;

    /// Take one step
    fn step(&mut self) -> bool;

    /// Replace self with some other value
    fn goto(&mut self, destination: &Self);
}

pub trait Random {
    //! A trait for things that can be searched through randomly. For example, the stochastic
    //! search uses this.

    /// Start from a random point
    fn random() -> Self;

    /// Take a step in a random direction
    fn step(&mut self);

    /// Replace self with some other value
    fn goto(&mut self, destination: &Self);
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

pub trait ConstraintSatisfaction<T> {
    //! A trait for constraint solvers
    /// Considers the `T` passed to this method, and checks if it violates any unary constraints.
    fn unary(&self, t: &T) -> ConstraintViolation<T>;

    /// Considers the two connected nodes of type `T`, and sees if they violate any binary
    /// constraints.
    fn binary(&self, a: &T, b: &T) -> ConstraintViolation<T>;
}
