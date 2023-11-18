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
    Some(a * b)
}

fn bruteforce_search(label: &'static str, func: fn(i32, i32) -> Option<i32>) {
    let program = strop::armv4t::thumb()
        .bruteforce_with_maximum_length(5)
        .aapcs32(func)
        .next()
        .unwrap();

    println!("{}:", label);
    program.disassemble();
    println!("\tmov pc, lr"); // this should do the trick.
}

fn stochastic_search(label: &'static str, func: fn(i32, i32) -> Option<i32>) {
    let program = strop::armv4t::thumb()
        .branchless()
        .stochastic_search()
        .aapcs32(func)
        .next()
        .unwrap();

    println!("{}:", label);
    program.disassemble();
    println!("\tmov pc, lr"); // this should do the trick.
}

fn main() {
    // These functions are so simple, mapping to a single ARM instruction, that a bruteforce search works okay.
    bruteforce_search("add", |x, y| x.checked_add(y));
    bruteforce_search("shl", |x, y| x.checked_shl(y as u32));
    bruteforce_search("shr", |x, y| x.checked_shr(y as u32));
    bruteforce_search("mul", |x, y| x.checked_mul(y));

    // These functions a bit more involved and need several instructions to compute.
    stochastic_search("salt", salt);
    stochastic_search("pepper", pepper);
}
