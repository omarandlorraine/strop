#[cfg(not(feature = "mips"))]
fn run_test() {
    unreachable!("the mips module has been configured out!");
}

#[cfg(feature = "mips")]
fn run_test() {
    use strop::ToBruteForce;

    fn identity(f: u16) -> strop::RunResult<u16> {
        Ok(f)
    }

    let mut search = strop::sm83::SdccCall1::<u16, u16>::default()
        .to_bruteforce(identity as fn(u16) -> strop::RunResult<u16>);

    while let Some(id) = search.search() {
        strop::Disassembly::print(&id, "identity");
    }
}

fn main() {
    println!("This program lists all possible ways to implement the identity function in SM83");
    println!("and is intended to give human developers an idea of missed heuristics");
    println!("such as peephole optimizations etc.");
    run_test();
}
