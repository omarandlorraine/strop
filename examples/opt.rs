//! An example of a program that uses strop to optimize an existing function.

use strop::z80::Insn;
use strop::z80::SdccCall1;
use strop::BruteForce;
use strop::Callable;
use strop::Disassemble;

fn target_function() -> SdccCall1 {
    // Construct some machine code.
    //
    // In a real world scenario maybe you'd read this in from assembly or something, but you can
    // also build up the machine code program in the way shown below:
    //
    // It's equivalent to this C code:
    // `uint16_t f(uint16_t unused) { return 16511; }`
    //
    // This is not a terribly efficient way to encode this program; you can save a byte and some
    // time with `LD HL, 7F40H` instead (a single 16-bit bit immediate load is more efficient than two
    // individual 8-bit loads) -- let's see if strop figures this out!
    //
    // When building a SdccCall1 callable we leave off the terminating RET instruction since when
    // SdccCall1 builds, it adds one

    use strop::Goto;
    use strop::IterableSequence;

    let mc = [
        Insn::new(&[0x26, 0x40]), // LD H,40H
        Insn::new(&[0x2e, 0x7f]), // LD L,7FH
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
    let mut c = SdccCall1::first();
    c.goto(&mc);
    c
}

fn main() {
    use strop::Iterable;
    let c = target_function();

    // you can call this function in a few different ways
    let result: u16 = c.call(5u16).unwrap();
    println!("The function returns {result}");

    println!("The subroutine we started with:");
    c.dasm();

    // you can do a bruteforce search for Z80 machine code programs implementing the same function
    let mut bruteforce: BruteForce<u16, u16, SdccCall1, _> =
        strop::BruteForce::new(c, SdccCall1::first());

    let bf = bruteforce.search().unwrap();

    println!("An equivalent subroutine we found by bruteforce search:");
    bf.dasm();
}
