//! An example of a program that uses strop to generate machine code computing a given function

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
    // do a bruteforce search for Z80 machine code programs implementing the same function
    let mut search = strop::bruteforce::BruteForce::new(
        zero as fn(u8) -> strop::RunResult<u8>,
        strop::sm83::SdccCall1::<u8, u8>::default(),
    );

     search.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search,");
    println!("after {} iterations.", search.count);
    search.dasm();
}
