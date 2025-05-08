pub fn identity(f: f32) -> strop::RunResult<f32> {
    Ok(f)
}

fn main() {
    use strop::Disassemble;
    use strop::ToBruteForce;

    println!("This program lists all possible ways to implement the identity function in MIPS");
    println!("and is intended to give human developers an idea of missed heuristics");
    println!("such as peephole optimizations etc.");

    let mut search =
        strop::mips::O32::default().to_bruteforce(identity as fn(f32) -> strop::RunResult<f32>);

    while let Some(id) = search.search() {
        println!("identity:");
        id.dasm();
    }
}
