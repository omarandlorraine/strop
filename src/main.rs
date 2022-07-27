extern crate clap;
use clap::clap_app;

mod machine;
mod search;
use crate::machine::mos6502::Instruction6502;
use crate::machine::stm8::Stm8Instruction;
use crate::machine::x80::KR580VM1Instruction;
use crate::search::BasicBlock;

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
    let matches = clap_app!(strop =>
        (version: "1.0")
        (author: "Sam M W <sam.magnus.wilson@gmail.com>")
        (about: "Stochastically generates machine code")
        (@subcommand kr580vm1 =>
         (about: "Generates code for the KR580VM1 or Intel 8080")
         (@arg only80: --only80 "do not permit KR580VM1 specific instructions (i.e., the generated program will be compatible with the Intel 8080)"))
        (@subcommand stm8 =>
         (about: "Generates code for the STM8"))
        (@subcommand mos6502 =>
         (about: "Generates code for the MOS 6502")
         (@arg rorbug: --rorbug "avoid the bug in the ROR instruction of very early chips")
         (@arg cmos: --cmos "allow CMOS instructions (including phx, stz)")
         (@arg illegal: --illegal "allow illegal instructions (including lax, dcp, anc)"))
        (@arg ins: -i --in +takes_value ... "where to find inputs to the function")
        (@arg outs: -o --out +takes_value ... "where to put the function's output")
    ).get_matches();

    match matches.subcommand() {
        ("kr580vm1", sub_matches) => {
            println!("Calling out to kr580vm1");
        }
        ("mos6502", sub_matches) => {
            if let Some(opts) = sub_matches {
                let cmos = opts.is_present("cmos");
                let illegal = opts.is_present("illegal");
                let rorbug = opts.is_present("rorbug");
                if cmos && rorbug {
                    println!("Don't specify --cmos and --rorbug together; there are no chips having both CMOS instructions and the ROR bug.");
                }
                if cmos && illegal {
                    println!("Don't specify --cmos and --illegal together; there are no chips having both CMOS instructions and NMOC illegal instructions.");
                }
            }
        }
        (_, _) => {
            panic!();
        }
    }
}
