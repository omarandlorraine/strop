//! An example of a program that uses strop to generate machine code computing a given function

use strop::Callable;
use strop::Disassemble;
use strop::RunError;
use strop::RunResult;

fn add5(i: u8) -> RunResult<u8> {
    i.checked_add(5).ok_or(RunError::NotDefined)
}

fn zero(_s: u8) -> crate::RunResult<u8> {
    Ok(b'0')
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let Some(triplet) = strop::triplets::Triplet::search(&args[1]) else {
        println!("triplet {:?} not found. Try one of these:", &args[1]);

        for triplet in strop::triplets::Triplet::all() {
            println!(" - {triplet:?}");
        }
        panic!("impossible to continue with no target triplet");
    };
    println!("building the function for {triplet:?}");

    // do a bruteforce search for the function in the target language
    let mut search = strop::bruteforce::BruteForce::new(
        add5 as fn(u8) -> strop::RunResult<u8>,
        strop::z80::SdccCall1::default(),
    );

    /*
    while !search.test() {
        search.step();
        println!("one step:");
        search.dasm();
    }
    */

    search.search().unwrap();

    println!("the following function was found by bruteforce search,");
    println!("after {} iterations.", search.count);
    search.dasm();
}
