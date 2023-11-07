use strop::InstructionSet;

// A program to generate a library of useful subroutines for the ARMv4T CPUs. The parameter passing
// convention matches my understanding of AAPCS32, so that the routines should be callable from C,
// but this has not been tested.

fn pepper(a: i32, _b: i32) -> Option<i32> {
    // Increments the lower 4 bits of a
    let inc = a & (0x0f + 1);
    Some((a & !0x0f) | (inc & 0x0f))
}

fn salt(a: i32, b: i32) -> Option<i32> {
    Some((a & 0x1f) | ((b & 0x0700) >> 3))
}

fn generate2(label: &'static str, func: fn(i32, i32) -> Option<i32>) {
    let program = strop::armv4t::thumb()
        .bruteforce_with_maximum_length(5)
        .aapcs32(func)
        .next()
        .unwrap();

    println!("{}:", label);
    program.disassemble();
    println!("\tmov pc, lr"); // this should do the trick.
}

fn main() {
    generate2("add", |x, y| x.checked_add(y));
    generate2("shl", |x, y| x.checked_shl(y as u32));
    generate2("shr", |x, y| x.checked_shr(y as u32));
    generate2("mul", |x, y| x.checked_mul(y));
    generate2("pepper", pepper);
    generate2("salt", salt);
}
