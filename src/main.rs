use std::fs;
use std::process;

extern crate argh;
use argh::FromArgs;

mod machine;
mod search;
mod test;

use crate::machine::Datum;
use crate::machine::State;
use crate::machine::{Machine, Mos6502Variant, Motorola8BitVariant, PicVariant, PreX86Variant};
use crate::search::stochastic_search;
use crate::search::BasicBlock;
use crate::search::{difference, optimize};

use crate::test::sanity;
use crate::test::{DeTestRun, Step, Test, TestRun};

struct MOpt {
    name: &'static str,
    mach: Machine,
    help: &'static str,
}

const M_OPTS: [MOpt; 14] = [
    MOpt {
        name: "8080",
        mach: Machine::PreX86(PreX86Variant::I8080),
        help: "Intel 8080",
    },
    MOpt {
        name: "kr580vm1",
        mach: Machine::PreX86(PreX86Variant::KR580VM1),
        help: "KR580VM1, a Soviet Ukrainian 8080 variant",
    },
    MOpt {
        name: "z80",
        mach: Machine::PreX86(PreX86Variant::ZilogZ80),
        help: "Zilog Z80",
    },
    MOpt {
        name: "sm83",
        mach: Machine::PreX86(PreX86Variant::Sm83),
        help: "Weirdo found in some nintendos",
    },
    MOpt {
        name: "2a03",
        mach: Machine::Mos6502(Mos6502Variant::Ricoh2a03),
        help: "Ricoh 2A03/2A07, which is a 6502 with no decimal mode",
    },
    MOpt {
        name: "6502",
        mach: Machine::Mos6502(Mos6502Variant::Nmos),
        help: "generic 6502",
    },
    MOpt {
        name: "65i02",
        mach: Machine::Mos6502(Mos6502Variant::IllegalInstructions),
        help: "NMOS 6502, but with illegal instructions like lax and dca",
    },
    MOpt {
        name: "65c02",
        mach: Machine::Mos6502(Mos6502Variant::Cmos),
        help: "CMOS 6502, including new instructions like phx and stz",
    },
    MOpt {
        name: "6800",
        mach: Machine::Motorola6800(Motorola8BitVariant::Motorola6800),
        help: "Motorola 6800",
    },
    MOpt {
        name: "6801",
        mach: Machine::Motorola6800(Motorola8BitVariant::Motorola6801),
        help: "Motorola 6801",
    },
    MOpt {
        name: "pic12",
        mach: Machine::Pic(PicVariant::Pic12),
        help: "PIC12",
    },
    MOpt {
        name: "pic14",
        mach: Machine::Pic(PicVariant::Pic14),
        help: "PIC14",
    },
    MOpt {
        name: "pic16",
        mach: Machine::Pic(PicVariant::Pic16),
        help: "PIC16",
    },
    MOpt {
        name: "stm8",
        mach: Machine::Stm8,
        help: "low-cost microcontroller family by STMicroelectronics",
    },
];

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

fn mach(m: String) -> Machine {
    for m_opt in &M_OPTS {
        if m_opt.name == m {
            return m_opt.mach;
        }
    }
    println!("You didn't pick a valid arch, so here's the ones I know:");
    for m_opt in &M_OPTS {
        println!("\t{:>8}  {}", m_opt.name, m_opt.help);
    }
    process::exit(1);
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

fn disassemble(prog: BasicBlock) {
    for p in prog.instructions {
        println!("{}", p);
    }
}

fn testrun_from_args(opts: &Opts, mach: Machine) -> TestRun {
    let ins: Vec<Datum> = opts
        .r#in
        .clone()
        .into_iter()
        .map(|reg| mach.register_by_name(&reg))
        .collect();
    let outs: Vec<Datum> = opts
        .out
        .clone()
        .into_iter()
        .map(|reg| mach.register_by_name(&reg))
        .collect();
    TestRun {
        tests: function(opts.function.clone().unwrap(), ins, outs),
    }
}

fn main() {
    let opts: Opts = argh::from_env();
    let machine = mach(opts.arch.clone());

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
