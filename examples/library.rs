use strop::InstructionSet;

// A program to generate a library of useful subroutines for the ARMv4T CPUs. The parameter passing
// convention matches my understanding of AAPCS32, so that the routines should be callable from C,
// but this has not been tested.

fn main() {
    let bruteforce = strop::armv4t::thumb()
        .bruteforce_with_maximum_length(5)
        .aapcs32(|x, y| x.checked_add(y));

    for candidate in bruteforce {
        candidate.disassemble();

        println!("*************");
    }
}
