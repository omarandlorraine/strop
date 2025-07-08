//! An example of a program that uses strop to generate machine code computing a given function

use strop::Disassemble;
use strop::RunError;
use strop::RunResult;
use strop::ToBruteForce;
use strop::ToTrace;
use strop::triplets::Triplet;

fn zero(i: u8) -> RunResult<u8> {
    i.checked_add(5).ok_or(RunError::NotDefined)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        eprintln!("This program requires you to specify a target triplet.");
        eprintln!("Here's a list of ones I know:");
        for triplet in Triplet::all() {
            eprintln!(" - {triplet}");
        }
    }

    let Some(triplet) = Triplet::search(&args[1]) else {
        eprintln!("No such target triplet as {}!", args[1]);
        eprintln!("Here's a list of ones I know:");
        for triplet in Triplet::all() {
            eprintln!(" - {triplet}");
        }
        return;
    };

    let target_function = zero as fn(u8) -> RunResult<u8>;

    // do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce = strop::mips::O32::default()
        .trace()
        .to_bruteforce(target_function);

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search,");
    println!("after {} iterations.", bruteforce.count);
    bf.dasm();
}
