use std::fs;
use std::process;

extern crate argh;
use argh::FromArgs;

mod machine;
mod search;
mod test;

use crate::machine::Datum;
use crate::machine::Machine;
use crate::machine::MACHINES;
use crate::search::optimize;
use crate::search::stochastic_search;
use crate::search::BasicBlock;

use crate::test::sanity;
use crate::test::{DeTestRun, Step, Test, TestRun};

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

    #[argh(option)]
    /// constants
    constant: Vec<i8>,

    #[argh(switch, short = 'g')]
    /// graph progress
    graph: bool,

    #[argh(switch, short = 'd')]
    /// disassemble the best specimen from each generation
    debug: bool,
}

fn function(m: String, ins: Vec<Datum>, outs: Vec<Datum>) -> Vec<Test> {
    // TODO: test_cases does not need to be mutable..
    let mut test_cases = Vec::new();
    if m == *"abs" {
        for n in -127_i8..=127 {
            test_cases.push(Test {
                steps: vec![
                    Step::Set(ins[0], n as i32),
                    Step::Run,
                    Step::Diff(outs[0], n.abs() as i32),
                ],
            });
        }
        return test_cases;
    }

    if m[0..4] == *"mult" {
        let arg = m[4..].to_string();
        let a = arg.parse::<i8>();

        if let Ok(f) = a {
            for n in -128_i8..=127 {
                if let Some(res) = n.checked_mul(f) {
                    test_cases.push(Test {
                        steps: vec![
                            Step::Set(ins[0], n as i32),
                            Step::Run,
                            Step::Diff(outs[0], res as i32),
                        ],
                    });
                }
            }
            return test_cases;
        } else {
            println!("Can't multiply by {}", arg);
        }
    }
    if m[0..3] == *"add" {
        let arg = m[3..].to_string();
        let a = arg.parse::<i8>();

        if let Ok(f) = a {
            for n in -128_i8..=127 {
                if let Some(res) = n.checked_add(f) {
                    test_cases.push(Test {
                        steps: vec![
                            Step::Set(ins[0], n as i32),
                            Step::Run,
                            Step::Diff(outs[0], res as i32),
                        ],
                    });
                }
            }
            return test_cases;
        } else {
            println!("Can't add {}", arg);
        }
    }
    if m[0..3] == *"max" {
        let arg = m[3..].to_string();
        let a = arg.parse::<i8>();

        if let Ok(f) = a {
            for n in -128_i8..=127 {
                test_cases.push(Test {
                    steps: vec![
                        Step::Set(ins[0], n as i32),
                        Step::Run,
                        Step::Diff(outs[0], if n < f { n } else { f } as i32),
                    ],
                });
            }
            return test_cases;
        } else {
            println!("Can't add {}", arg);
        }
    }
    println!("I don't understand what you mean by the argument {}", m);
    process::exit(1);
}

fn disassemble(prog: BasicBlock<State, Operand, OUD, IUD>) {
    for p in prog.instructions {
        println!("{}", p);
    }
}

fn testrun_from_args(opts: &Opts, mach: Machine) -> TestRun {
    for l in opts.r#in.clone().into_iter().chain(opts.out.clone()) {
        if let Some(error_message) = mach.register_by_name(&l).err() {
            println!(
                "I don't understand what you mean by the datum {}: {}.",
                l, error_message
            );
            process::exit(1);
        }
    }

    let ins: Vec<Datum> = opts
        .r#in
        .clone()
        .into_iter()
        .map(|reg| mach.register_by_name(&reg).unwrap())
        .collect();
    let outs: Vec<Datum> = opts
        .out
        .clone()
        .into_iter()
        .map(|reg| mach.register_by_name(&reg).unwrap())
        .collect();
    TestRun {
        tests: function(opts.function.clone().unwrap(), ins, outs),
    }
}

fn get_machine_by_name(name: &str) -> Machine {
    for m in MACHINES {
        if m.name == name {
            return m;
        }
    }

    println!("I don't know a machine called {}", name);
    println!("Here are the ones I know:");
    for m in MACHINES {
        println!(" - {}", m.name);
    }
    process::exit(1);
}

fn main() {
    let opts: Opts = argh::from_env();

    let machine = get_machine_by_name(&opts.arch);

    let testrun = if let Some(path) = opts.file {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let res: DeTestRun = serde_json::from_str(&data).expect("Unable to parse");
        sanity(&res, machine)
    } else {
        testrun_from_args(&opts, machine)
    };

    let prog = stochastic_search(&testrun, machine, opts.graph, opts.debug);
    let opt = optimize(&testrun, &prog, machine);
    disassemble(opt);
}
