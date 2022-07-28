extern crate clap;
use clap::{Arg, ArgAction, Command};

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
    let matches = Command::new("strop")
        .version("0.1.0")
        .author("Sam M W <sam.magnus.wilson@gmail.com>")
        .about("Stochastically generates machine code")
        .subcommand(
            Command::new("kr580vm1").about("Generates code for the KR580VM1 or Intel 8080")
            .arg(Arg::new("only80").action(ArgAction::SetTrue)).about("do not permit KR580VM1 specific instructions (i.e., the generated program will be compatible with the Intel 8080)"))
        .subcommand(
            Command::new("stm8").about("Generates code for the STM8"))
        .subcommand(
            Command::new("mos6502").about("Generates code for the MOS 6502")
            .arg(Arg::new("rorbug").action(ArgAction::SetTrue)).about("avoid the bug in the ROR instruction of very early chips")
            .arg(Arg::new("cmos").action(ArgAction::SetTrue)).about("allow CMOS instructions (including phx, stz)")
            .arg(Arg::new("illegal").action(ArgAction::SetTrue)).about("allow illegal instructions (including lax, dcp, anc)")
        )
        .arg(Arg::new("in").short('i').help("where to find inputs to the function").action(ArgAction::Append))
        .arg(Arg::new("out").short('o').help("where to put the function's outputs").action(ArgAction::Append))
    .get_matches();

    match matches.subcommand() {
        Some(("kr580vm1", opts)) => {
            println!("Calling out to kr580vm1");
        }
        Some(("mos6502", opts)) => {
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
        Some((_, _)) => {
            panic!();
        }
        None => {
            panic!();
        }
    }
}
