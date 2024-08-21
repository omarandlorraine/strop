//! An example of a program that uses strop to optimize an existing function.

use strop::z80::Insn;
use strop::z80::SdccCall1;

fn main() {
    // Construct the existing machine code.
    let mc = [
        Insn::new(&[0x06, 0x40]), // LD B,40H
        Insn::new(&[0x0e, 0x7f]), // LD C,7FH
        Insn::new(&[0xc9]),       // RET
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
}
