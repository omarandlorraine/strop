//! Superoptimizer written in Rust

//! This program stochastically generates assembly language programs that compute a given function.
//! The idea is you give it a function to compute and specify which registers and things to use,
//! and strop will generate and output a pretty good program which does the specified thing.
//!
//! So far, strop has had a focus on supporting architectures that are not well supported by
//! mainstream compilers such as LLVM. These have included architectures common in low-end
//! embedded, and hobbyist retrocomputing.

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
pub mod machine;
pub mod search;
