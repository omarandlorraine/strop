use strop::BruteForce;
use strop::Disassemble;
use strop::Iterable;
use strop::StropError;

fn zero(_nothing: u8) -> Result<u8, StropError> {
    Ok(b'0')
}

fn check() {
    use strop::z80::Constraints;
    use strop::z80::Insn;
    use strop::z80::SdccCall1;
    use strop::Constrain;
    use strop::Goto;

    let mc = [
        Insn::new(&[0x3e, 0x30]), // ld a, 30h
        Insn::new(&[0x10, 0x00]), // djnz 0
    ];

    // This machine code is callable using the sdcccall(1) calling convention.
    let mut c: SdccCall1<u8, u8> = SdccCall1::first();
    c.goto(&mc);

    strop::report(&c.build(), &Constraints::default().basic_block());
}

fn sdcccall1_search(target_function: fn(u8) -> Result<u8, StropError>) {
    use strop::z80::SdccCall1;

    let target_function = target_function as fn(u8) -> Result<u8, StropError>;

    // a bruteforce search for Z80 machine code programs implementing the function
    let mut bruteforce: BruteForce<_, _, _, SdccCall1<_, _>> =
        BruteForce::new(target_function, SdccCall1::first());

    // let's find the first program that implements the function!
    let first = bruteforce.search().unwrap();

    let mut count = 0usize;

    // let's find more programs that are equivalent. I'm expecting these to have some
    // inefficiencies, which will point out deficiencies in the peephole optimizers and dataflow
    // analysis.
    loop {
        println!("about to step");
        if !bruteforce.step() {
            break;
        }
        println!("stepped");

        /*
        if bruteforce.candidate().len() > first.len() + 1 {
            break;
        }
        */

        println!("\n\nunder consideration:");
        bruteforce.candidate().dasm();

        if !bruteforce.test() {
            println!("test failes");
            continue;
        }
        println!("test passed");

        if count == 0 {
            println!(
                "I've discovered two programs that are equivalent. One's going to have dead code"
            );
            println!("or some other inefficency.");

            println!("first:");
            //first.dasm();
            count = 1;
        }

        println!("next_{count}");
        bruteforce.candidate().dasm();
    }
}

fn main() {
    check();
    sdcccall1_search(zero);
}
