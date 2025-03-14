//! An example of a program that uses strop to generate machine code computing a given function

use strop::AsBruteforce;
use strop::Disassemble;
use strop::RunError;
use strop::RunResult;
use strop::ToTrace;

fn zero(_hex: u8) -> RunResult<u8> {
    Ok(b'0')
}

fn dec_to_hex(hex: u8) -> RunResult<u8> {
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
        _ => Err(RunError::NotDefined),
    }
}

fn main() {
    let target_function = zero as fn(u8) -> RunResult<u8>;

    // do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce = strop::z80::SdccCall1::default()
        .trace()
        .bruteforce(target_function);

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search,");
    println!("after {} iterations.", bruteforce.count);
    bf.dasm();
}
