#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

// This macro was shamelessly stolen from
// https://github.com/BartMassey/randomly/

/*!
Macro to select a random block. This is useful in games,
where taking random actions is common.

# Examples

```
use strop::randomly;

let n = randomly! {
    { println!("hello"); 0 }
    { println!("goodbye"); 1 }
};
println!("chose {}", n);
```
*/
use pyo3::prelude::*;

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

#[test]
fn test_randomly_inner() {
    let k = randomly!(@ 0, ({0}), ());
    assert_eq!(k, 0);
}

#[test]
fn test_randomly() {
    // XXX This test will fail with probability 1/3**100. I
    // can live with that.
    let mut changed = false;
    let mut last = randomly! {
        { 1 }
        { 2 }
        { 3 }
    };
    for _ in 0..100 {
        let n = randomly! {
            { 1 }
            { 2 }
            { 3 }
        };
        assert!((1u8..=3).contains(&n));
        if n != last {
            changed = true;
            last = n;
        }
    }
    assert!(changed);
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn string_sum(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
