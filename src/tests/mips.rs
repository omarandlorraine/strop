#[cfg(not(feature = "mips"))]
fn run_test() {
    unreachable!("the mips module has been configured out!");
}

#[cfg(feature = "mips")]
fn run_test() {
    use strop::Disassemble;
    use strop::ToBruteForce;
    use strop::ToTrace;

    fn identity(f: f32) -> strop::RunResult<f32> {
        Ok(f)
    }

    let mut search = strop::mips::O32::default()
        .trace()
        .to_bruteforce(identity as fn(f32) -> strop::RunResult<f32>);

    while let Some(id) = search.search() {
        println!("identity:");
        id.dasm();
    }
}

fn main() {
    println!("This program lists all possible ways to implement the identity function in MIPS");
    println!("and is intended to give human developers an idea of missed heuristics");
    println!("such as peephole optimizations etc.");
    run_test();
}
