//! An example of a program that uses strop to generate machine code computing a given function

use strop::Disassemble;
use strop::RunError;
use strop::RunResult;
use strop::ToBruteForce;
use strop::ToTrace;

fn zero(i: u8) -> RunResult<u8> {
    i.checked_add(5).ok_or(RunError::NotDefined)
}

fn main() {
    let target_function = zero as fn(u8) -> RunResult<u8>;

    // do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce = strop::z80::SdccCall1::default()
        .trace()
        .to_bruteforce(target_function);

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search,");
    println!("after {} iterations.", bruteforce.count);
    bf.dasm();
}
