use strop::Disassemble;
use strop::StropError;

fn zero(_nothing: u8) -> Result<u8, StropError> {
    Ok(b'0')
}

fn sdcccall1_search(target_function: fn(u8) -> Result<u8, StropError>) {
    use strop::z80::SdccCall1;

    let target_function = target_function as fn(u8) -> Result<u8, StropError>;

    // a bruteforce search for Z80 machine code programs implementing the function
    let mut bruteforce = SdccCall1::<u8, u8>::new()
        // By specifying that we want a pure function, and that the function is a leaf function, we
        // can constrain the search space even further
        .pure()
        .leaf()
        .bruteforce(target_function);

    // let's find the first program that implements the function!
    let first = bruteforce.search().unwrap();

    println!("found first:");
    first.dasm();

    let mut count = 0usize;

    // let's find more programs that are equivalent. I'm expecting these to have some
    // inefficiencies, which will point out deficiencies in the peephole optimizers and dataflow
    // analysis.
    loop {
        let second = bruteforce.search().unwrap();

        if count == 1 {
            println!(
                "I've discovered two or more programs that are equivalent. One's going to have dead code"
                );
            println!("or some other inefficency.");
        }

        println!("number {count}:");
        second.dasm();
        count += 1;

    }
}

fn main() {
    sdcccall1_search(zero);
}
