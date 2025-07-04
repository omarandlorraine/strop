#[cfg(not(feature = "m68k"))]
fn run_test() {
    unreachable!("the m68k module has been configured out!");
}

#[cfg(feature = "m68k")]
fn run_test() {
    use strop::Disassemble;
    use strop::ToBruteForce;
    use strop::ToTrace;

    fn identity(f: u32) -> strop::RunResult<u32> {
        Ok(f)
    }

    let mut search = strop::m68k::Regparm::default()
        .trace()
        .to_bruteforce(identity as fn(u32) -> strop::RunResult<u32>);

    while let Some(id) = search.search() {
        println!("identity:");
        id.dasm();
    }
}

fn main() {
    println!("This program lists all possible ways to implement the identity function in M68000 assembly");
    println!("and is intended to give human developers an idea of missed heuristics");
    println!("such as peephole optimizations etc.");
    run_test();
}
