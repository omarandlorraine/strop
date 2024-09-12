//! An example of a program that uses strop to optimize an existing function.

use strop::z80::Insn;
use strop::z80::SdccCall1;
use strop::z80::Subroutine;
use strop::BruteForce;
use strop::Callable;
use strop::Disassemble;

fn main() {
    use strop::z80::IntoSubroutine;

    // Construct some machine code.
    // It's equivalent to this C code:
    // `int f(uint16_t unused) { return 16511; }`
    //
    // (this is not a terribly efficient way to encode this program; you can save a byte and some
    // time with hex 217f40c9 instead -- let's see if strop figures this out!)
    let mc = [
        Insn::new(&[0x26, 0x40]), // LD H,40H
        Insn::new(&[0x2e, 0x7f]), // LD L,7FH
        Insn::new(&[0xc9]),       // RET
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
    let c = SdccCall1::into_subroutine(&mc);

    // you can call this function
    println!("The function returns {}", c.call(5).unwrap());

    println!("The subroutine we started with:");
    c.dasm();

    // you can do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce: BruteForce<
        _,
        _,
        Subroutine<u16, u16, SdccCall1>,
        Subroutine<u16, u16, SdccCall1>,
    > = strop::BruteForce::new(c);

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search:");
    bf.dasm();
}
