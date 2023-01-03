use argh::FromArgs;
use rand::random;
use strop::machine::mos6502::{Instruction6502, Mos6502};
use strop::search::stochastic_search;
use strop::search::BasicBlock;
use strop::Search;

#[derive(FromArgs)]
/// multiplies the accumulator by a constant
struct Cli {
    /// turn verbose on
    #[argh(switch, short = 'v')]
    verbose: bool,

    #[argh(positional, description = "multiply by this number")]
    multiply_by: u8,
}

struct MultiplyByConstant {
    constant: u8,
}

impl Search<Instruction6502> for MultiplyByConstant {
    fn optimize(&self, bb: &BasicBlock<Instruction6502>) -> f64 {
        bb.len() as f64
    }

    fn correctitude(&self, bb: &BasicBlock<Instruction6502>) -> f64 {
        let factor = self.constant;
        use strop::machine::Instruction;

        let mut state: Mos6502 = Default::default();
        let mut error: f64 = 0.0;

        for _i in 0..1000 {
            let input: u8 = random();
            if let Some(result) = input.checked_mul(factor) {
                state.carry = Some(false);
                state.decimal = Some(false);
                state.a = Some(input);

                for insn in &bb.instructions {
                    insn.operate(&mut state);
                }

                if let Some(a) = state.a {
                    error += (f64::from(a) - f64::from(result)).abs();
                } else {
                    error += 1000.0;
                }
            }
        }

        error
    }
}

fn main() {
    let cli: Cli = argh::from_env();

    if cli.verbose {
        println!("A program to multiply by {}", cli.multiply_by);
    }

    let mul = MultiplyByConstant {
        constant: cli.multiply_by,
    };

    let prog = stochastic_search(mul);

    for insn in &prog.instructions {
        println!("{}", insn);
    }
}
