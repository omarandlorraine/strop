extern crate argh;
use argh::FromArgs;

mod machine;
mod search;
use crate::machine::mos6502::Instruction6502;
use crate::machine::stm8::Stm8Instruction;
use crate::machine::x80::KR580VM1Instruction;
use crate::search::BasicBlock;

#[derive(FromArgs, PartialEq, Debug)]
/// command line arguments
struct Opts {
    #[argh(option, short = 'm')]
    /// the name of the architecture.
    arch: String,

    #[argh(option, short = 'f')]
    /// file containing the custom test run
    file: Option<String>,

    #[argh(option)]
    /// the function to compute
    function: Option<String>,

    #[argh(option, long = "in")]
    /// in variables
    r#in: Vec<String>,

    #[argh(option)]
    /// out variables
    out: Vec<String>,
}

fn disassemble<I: machine::Instruction + std::fmt::Display>(prog: BasicBlock<I>) {
    for p in prog.instructions {
        println!("{}", p);
    }
}

fn stocsearch<I: machine::Instruction>() {
    let prog = crate::search::stochastic_search::<I>(|bb| 0.0);
    disassemble::<I>(prog);
}

fn main() {
    let opts: Opts = argh::from_env();

    if &opts.arch == &"stm8" {
        stocsearch::<Stm8Instruction>();
    } else if &opts.arch == &"mos6502" {
        stocsearch::<Instruction6502>();
    } else if &opts.arch == &"kr580vm1" {
        stocsearch::<KR580VM1Instruction>();
    }
}
