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

pub use rand;

/// Randomly select a block to be executed. Blocks have
/// equal probability of being selected (that is, selection
/// is uniform random).
///
/// # Panics
///
/// Panics if zero blocks were provided.
// Lots of ideas borrowed from here:
// https://users.rust-lang.org/t/how-to-generate-in-macro/56774/6
#[macro_export]
macro_rules! randomly {
    (@ $n:expr, ($action:block $($rest:block)*), ($($arms:tt,)*)) => {
        randomly!(@ $n + 1, ($($rest)*), (($n, $action), $($arms,)*))
    };
    (@ $n:expr, (), ($(($m:expr, $action:block),)*)) => {{
        use $crate::rand::{thread_rng, Rng};
        let i: u32  = thread_rng().gen_range(0..$n);
        match i {
            $(x if x == $m => $action)*
            _ => panic!(),
        }
    }};
    ($($action:block)*) => {
        randomly!(@ 0, ($($action)*), ())
    };
}
