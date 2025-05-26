#[cfg(not(feature = "z80"))]
fn run_test() {
    unreachable!("the mips module has been configured out!");
}

#[cfg(feature = "z80")]
fn run_test() {
    use strop::Disassemble;
    use strop::ToBruteForce;

    fn identity(f: u16) -> strop::RunResult<u16> {
        Ok(f)
    }

    let mut search = strop::z80::SdccCall1::default()
        .to_bruteforce(identity as fn(u16) -> strop::RunResult<u16>);

    while let Some(id) = search.search() {
        println!("identity: ; (after {} iterations)", search.count);
        id.dasm();
    }
}

fn main() {
    println!("This program lists all possible ways strop finds to implement the identity");
    println!("function in Z80 assembler using the SDCCCALL(1) calling convention.");
    println!("This is intended to give human developers an idea of missed heuristics");
    println!("such as peephole optimizations etc.");
    run_test();
}
