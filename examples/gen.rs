//! An example of a program that uses strop to optimize an existing function.

use strop::z80::SdccCall1;
use strop::BruteForce;
use strop::Disassemble;
use strop::Iterable;
use strop::StropError;

/*
fn zero(_hex: u8) -> Result<u8, StropError> {
    Ok(b'0')
}
*/

fn target_function(hex: u8) -> Result<u8, StropError> {
    match hex {
        0x0 => Ok(b'0'),
        0x1 => Ok(b'1'),
        0x2 => Ok(b'2'),
        0x3 => Ok(b'3'),
        0x4 => Ok(b'4'),
        0x5 => Ok(b'5'),
        0x6 => Ok(b'6'),
        0x7 => Ok(b'7'),
        0x8 => Ok(b'8'),
        0x9 => Ok(b'9'),
        0xa => Ok(b'a'),
        0xb => Ok(b'b'),
        0xc => Ok(b'c'),
        0xd => Ok(b'd'),
        0xe => Ok(b'e'),
        0xf => Ok(b'f'),
        _ => Err(StropError::Undefined),
    }
}

fn main() {
    let target_function = target_function as fn(u8) -> Result<u8, StropError>;

    // you can do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce: BruteForce<_, _, _, SdccCall1<u8, u8>> =
        strop::BruteForce::new(target_function, SdccCall1::first());

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search:");
    bf.dasm();
}
