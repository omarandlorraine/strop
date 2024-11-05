use strop::BruteForce;
use strop::Disassemble;
use strop::Iterable;
use strop::StropError;

fn zero(_nothing: u8) -> Result<u8, StropError> {
    Ok(b'0')
}

fn sdcccall1_search(target_function: fn(u8) -> Result<u8, StropError>) {
    use strop::z80::SdccCall1;

    let target_function = target_function as fn(u8) -> Result<u8, StropError>;

    // a bruteforce search for Z80 machine code programs implementing the function
    let mut bruteforce: BruteForce<_, _, _, SdccCall1> =
        BruteForce::new(target_function, SdccCall1::first());

    // let's find the first program that implements the function!
    let Some(first) = bruteforce.search() else {
        return;
    };
    first.dasm();

    let mut count = 0usize;

    // let's find more programs that are equivalent. I'm expecting these to have some
    // inefficiencies, which will point out deficiencies in the peephole optimizers and dataflow
    // analysis.
    loop {
        if !bruteforce.step() {
            break;
        }

        if bruteforce.candidate().len() > first.len() + 1 {
            break;
        }

        if !bruteforce.test() {
            continue;
        }

        if count == 0 {
            println!("I've discovered two programs that are equivalent. One's going to have dead code");
            println!("or some other inefficency.");

            println!("first:");
            first.dasm();
            count = 1;
        }

        println!("next_{count}");
        bruteforce.candidate().dasm();
    }
}

fn main() {
    sdcccall1_search(zero);
}
