//! An example of a program that uses strop to optimize an existing function.

use strop::z80::SdccCall1;
use strop::BruteForce;
use strop::Disassemble;
use strop::Iterable;
use strop::StropError;

fn zero(hex: u8) -> Result<u8, StropError> {
    Ok('0' as u8)
}

fn target_function(hex: u8) -> Result<u8, StropError> {
    match hex {
        0x0 => Ok('0' as u8),
        0x1 => Ok('1' as u8),
        0x2 => Ok('2' as u8),
        0x3 => Ok('3' as u8),
        0x4 => Ok('4' as u8),
        0x5 => Ok('5' as u8),
        0x6 => Ok('6' as u8),
        0x7 => Ok('7' as u8),
        0x8 => Ok('8' as u8),
        0x9 => Ok('9' as u8),
        0xa => Ok('a' as u8),
        0xb => Ok('b' as u8),
        0xc => Ok('c' as u8),
        0xd => Ok('d' as u8),
        0xe => Ok('e' as u8),
        0xf => Ok('f' as u8),
        _ => Err(StropError::Undefined),
    }
}

fn main() {
    let target_function = zero as fn(u8) -> Result<u8, StropError>;

    // you can do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce: BruteForce<_, _, _, SdccCall1> =
        strop::BruteForce::new(target_function, SdccCall1::first());

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search:");
    bf.dasm();
}
