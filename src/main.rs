extern crate clap;
use clap::Arg;
use clap::Command;

pub mod machine;
pub mod search;

fn main() {
    let matches = Command::new("strop")
        .about("Stochastically generates machine code snippets")
        .version("0.1.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("Sam M W")
        .subcommand(
            Command::new("mult8")
            .about("Multiply an eight-bit value by a constant. The result is also eight bits, so overflows are undefined.")
            .arg_required_else_help(true)
            .arg(Arg::new("factor").help("multiply by this number").required(true))
            .arg(Arg::new("machine").help("target machine").required(true))
            .arg(Arg::new("source").help("what to multiply").required(true))
            .arg(Arg::new("destination").help("where do you want the result").required(true))).get_matches();

    println!("Nothing here yet.");
}
