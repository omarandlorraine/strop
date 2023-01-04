//! Superoptimizer written in Rust
//! ------------------------------
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

use search::BasicBlock;

pub use rand;

/// Implement this trait to define and constrain the search space.
pub trait Search<I: machine::Instruction> {
    /// Used to determine how "correct" a proposed program is (the return value 0 means, the
    /// program is correct, and the higher the return value, the more wrong the program is.)
    /// `stochastic_search` will halt when this function returns zero.
    fn correctitude(&self, prog: &BasicBlock<I>) -> f64;

    /// Default implementation of `optimize`. This implementation biases the search toward shorter
    /// programs, and so optimizes for size.
    fn optimize(&self, prog: &BasicBlock<I>) -> f64 {
        prog.instructions
            .iter()
            .map(|i| i.length() as u32)
            .sum::<u32>()
            .into()
    }

    /// Default implementation of `okay`. This implementation just returns True, which enables all
    /// instructions. I would expect end-users will want to use this to disqualify certain classes
    /// of instructions, such as those requiring specific hardware, or perhaps conditional
    /// branches, return-from-interrupt and the like.
    fn okay(&self, _: &I) -> bool {
        true
    }
}

// The reason I can't pull in randomly! as a dependency is that crates.io seems to require all my
// dependencies to also be on crates.io.

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
        let i = thread_rng().gen_range(0..$n);
        match i {
            $(x if x == $m => $action)*
            _ => panic!(),
        }
    }};
    ($($action:block)*) => {
        randomly!(@ 0, ($($action)*), ())
    };
}
