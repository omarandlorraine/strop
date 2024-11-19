//! An example of a program that uses strop to optimize an existing function.

use strop::z80::Insn;
use strop::z80::SdccCall1;

fn main() {
    use strop::Iterable;
    // Construct some machine code.
    //
    // In a real world scenario maybe you'd read this in from assembly or something, but you can
    // also build up the machine code program in the way shown below:
    //
    // It's equivalent to this C code:
    // `uint16_t f(uint16_t unused) { return 16511; }`
    //
    // To demonstrate strop's static analysis, this is a terribly inefficient way to encode this
    // program; it contains dead code, obvious opportunities for peephole optimization, incorrect
    // return instruction, etc.

    use strop::Goto;

    let mc = [
        Insn::new(&[0x06, 0x40]), // LD B,40H, this is dead code
        Insn::new(&[0x26, 0x40]), // LD H,40H
        Insn::new(&[0x2e, 0x7f]), // LD L,7FH, should just use `LD HL,` which is more efficient
        Insn::new(&[0x01, 0x00, 0x00]),
        Insn::new(&[0xd8]), // RET C
        Insn::new(&[0x00]),
        Insn::new(&[0xd0]), // RET NC, wrong way to return from a subroutine.
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
    let mut c = SdccCall1::<u16, u16>::first().peephole_optimization();
    c.goto(&mc);
    strop::report(&c, &c);

    let mc = [
        Insn::new(&[0xde, 0xc3]), // INC BC
        Insn::new(&[0x3e, 0x02]), // INC BC
        Insn::new(&[0xc9]),       // RET
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
    let mut c = SdccCall1::<u8, u8>::first().peephole_optimization();
    c.goto(&mc);
    strop::report(&c, &c);
}
