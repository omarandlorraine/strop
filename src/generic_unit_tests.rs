//! This module contains handy things that can be called from unit tests.

use crate::Instruction;

/// Lists all the instructions that disassemble to the given string.
///
/// Panics if the number of such instructions is less than or greater than 1.
pub fn list_all_encodings<I: Instruction + Clone>(dasm: &str, from: I, to: Option<I>) {
    let mut decodes = vec![];

    let mut a = from.clone();

    loop {
        if format!("{a}") == dasm {
            decodes.push(a.clone());
        }
        if a.increment().is_err() {
            break;
        }
        if let Some(ref to) = to {
            if a.to_bytes() == to.to_bytes() {
                break;
            }
        }
    }

    println!("All the ways to encode the instruction {dasm:?}:");
    for i in decodes.iter() {
        println!("{i:?}");
    }

    assert!(decodes.len() == 1);
}

/// Panics if the instruction type visits the same instruction twice.
///
/// This works by putting the disassembly in a hashset, and making sure not to put the same value
/// twice.
pub fn disassemblies_unique<I: Instruction>(from: I, to: Option<I>) {
    use std::collections::HashSet;

    let mut map: HashSet<String> = HashSet::new();

    let mut a = from;
    map.insert(format!("{a}"));

    while a.increment().is_ok() {
        let dasm = format!("{a}");

        if map.contains(&dasm) {
            panic!("{a} has been visited once before!");
        }
        if let Some(ref to) = to {
            if a.to_bytes() == to.to_bytes() {
                break;
            }
        }
    }
}

/// Panics if the instruction type visits the same instruction twice.
///
/// This works by doing a quadratic search between `i` and the end of the search space. Much slower
/// than `disassemblies_unique`, but doesn't take as much memory.
pub fn disassemblies_unique_slow<I: Instruction>(from: I, to: I) {
    let mut a = from;

    while a.increment().is_ok() {
        let mut b = I::from_bytes(&a.to_bytes()).unwrap();

        while b.increment().is_ok() {
            assert_ne!(format!("{a}"), format!("{b}"));
            if b.to_bytes() == to.to_bytes() {
                break;
            }
        }
        if a.to_bytes() == to.to_bytes() {
            break;
        }
    }
}

/// A few sanity checks for each instruction in the intstruction set
pub fn sanity_checks<I: Instruction>() {
    let mut a = I::first();

    loop {
        let dasm = format!("{a}"); // can disassemble

        let copy = I::from_bytes(&a.to_bytes()).unwrap();
        assert_eq!(a.to_bytes(), copy.to_bytes(), "{dasm}");
        assert_eq!(dasm, format!("{copy}"), "{dasm}");

        if a.increment().is_err() {
            // got to the end of the instruction set
            break;
        }
    }
}
