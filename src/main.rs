use std::fs;
use std::process;

extern crate argh;
use argh::FromArgs;

mod machine;
mod search;
mod test;

use crate::machine::motorola6800;
use crate::machine::{i8080, i8085, z80};
use crate::machine::{Machine, PreX86Variant, Mos6502Variant, Motorola8BitVariant, PicVariant};
use crate::machine::{mos6502, mos65c02};
use crate::machine::{pic12};

use crate::machine::Instruction;
use crate::machine::State;
use crate::machine::Datum;

use crate::search::BasicBlock;
use crate::search::{differance, equivalence};
use crate::search::{optimize, stochastic_search};

use crate::test::{sanity, DeTestRun, Parameter, Test, TestRun};

struct MOpt {
    name: &'static str,
    mach: Machine,
    func: fn() -> Vec<Instruction>,
    help: &'static str,
}

const M_OPTS: [MOpt; 13] = [
    MOpt {
        name: "8080",
        mach: Machine::PreX86(PreX86Variant::I8080),
        func: i8080,
        help: "Intel 8080",
    },
    MOpt {
        name: "8085",
        mach: Machine::PreX86(PreX86Variant::I8085),
        func: i8085,
        help: "Intel 8085",
    },
    MOpt {
        name: "kr580vm1",
        mach: Machine::PreX86(PreX86Variant::KR580VM1),
        func: z80,
        help: "KR580VM1, a Soviet Ukrainian 8080 variant",
    },
    MOpt {
        name: "z80",
        mach: Machine::PreX86(PreX86Variant::ZilogZ80),
        func: z80,
        help: "Zilog Z80",
    },
    MOpt {
        name: "2a03",
        mach: Machine::Mos6502(Mos6502Variant::Ricoh2a03),
        func: mos6502,
        help: "Ricoh 2A03/2A07, which is a 6502 with no decimal mode",
    },
    MOpt {
        name: "6502",
        mach: Machine::Mos6502(Mos6502Variant::Nmos),
        func: mos6502,
        help: "generic 6502",
    },
    MOpt {
        name: "65i02",
        mach: Machine::Mos6502(Mos6502Variant::IllegalInstructions),
        func: mos65c02,
        help: "NMOS 6502, but with illegal instructions like lax and dca",
    },
    MOpt {
        name: "65c02",
        mach: Machine::Mos6502(Mos6502Variant::Cmos),
        func: mos65c02,
        help: "CMOS 6502, including new instructions like phx and stz",
    },
    MOpt {
        name: "6800",
        mach: Machine::Motorola6800(Motorola8BitVariant::Motorola6800),
        func: motorola6800,
        help: "Motorola 6800",
    },
    MOpt {
        name: "6801",
        mach: Machine::Motorola6800(Motorola8BitVariant::Motorola6801),
        func: motorola6800,
        help: "Motorola 6800",
    },
    MOpt {
        name: "pic12",
        mach: Machine::Pic(PicVariant::Pic12),
        func: pic12,
        help: "PIC12",
    },
    MOpt {
        name: "pic14",
        mach: Machine::Pic(PicVariant::Pic14),
        func: pic12,
        help: "PIC14",
    },
    MOpt {
        name: "pic16",
        mach: Machine::Pic(PicVariant::Pic16),
        func: pic12,
        help: "PIC16",
    },
];

#[derive(FromArgs, PartialEq, Debug)]
/// Specify the machine you want to generate code for.
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

    #[argh(option)]
    /// what kind of search to perform
    search: String,

    #[argh(option, long = "in")]
    /// in variables
    r#in: Vec<String>,

    #[argh(option)]
    /// out variables
    out: Vec<String>,

    #[argh(option)]
    /// constants
    constant: Vec<i8>,
}

fn mach(m: String) -> (Vec<Instruction>, Machine) {
    for m_opt in &M_OPTS {
        if m_opt.name == m {
            return ((m_opt.func)(), m_opt.mach);
        }
    }
    println!("You didn't pick a valid arch, so here's the ones I know:");
    for m_opt in &M_OPTS {
        println!("\t{}  {}", format!("{:>8}", m_opt.name), m_opt.help);
    }
    process::exit(1);
}

fn function(m: String) -> Vec<Test> {
    // TODO: test_cases does not need to be mutable..
    let mut test_cases = Vec::new();
    if m == *"id" {
        for n in -128..=127 {
            test_cases.push(Test {
                ins: vec![n],
                outs: vec![n],
            });
        }
        return test_cases;
    }
    if m == *"signum" {
        for n in -128..=-1 {
            test_cases.push(Test {
                ins: vec![n],
                outs: vec![-1],
            });
        }
        test_cases.push(Test {
            ins: vec![0],
            outs: vec![0],
        });
        for n in 1..=127 {
            test_cases.push(Test {
                ins: vec![n],
                outs: vec![1],
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
                        ins: vec![n],
                        outs: vec![res],
                    });
                }
            }
            return test_cases;
        } else {
            println!("Can't multiply by {}", arg);
        }
    }
    if m[0..4] == *"idiv" {
        let arg = m[4..].to_string();
        let a = arg.parse::<i8>();

        if let Ok(f) = a {
            for n in 0..=127 {
                test_cases.push(Test {
                    ins: vec![n],
                    outs: vec![n / f],
                });
            }
            return test_cases;
        } else {
            println!("Can't divide by {}", arg);
        }
    }
    if m[0..3] == *"add" {
        let arg = m[3..].to_string();
        let a = arg.parse::<i8>();

        if let Ok(f) = a {
            for n in -128_i8..=127 {
                if let Some(res) = n.checked_add(f) {
                    test_cases.push(Test {
                        ins: vec![n],
                        outs: vec![res],
                    });
                }
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

fn constants(c: Vec<i8>) -> Vec<i8> {
    let mut v = Vec::<i8>::new();
    for i in 0..16 {
        v.push(i);
        v.push(0 - i);
    }
    for i in 0..7 {
        v.push(2i8.pow(i));
    }
    c.into_iter().chain(v).collect()
}

fn testrun_from_args(opts: &Opts, mach: Machine) -> TestRun {
    TestRun {
        ins: opts
            .r#in
            .clone()
            .into_iter()
            .map(|reg| Parameter{register: mach.register_by_name(&reg), cost: Some(0.0), address: None, name: reg.clone()})
            .collect(),
        outs: opts
            .out
            .clone()
            .into_iter()
            .map(|reg| Parameter{register: mach.register_by_name(&reg), cost: Some(0.0), address: None, name: reg.clone()})
            .collect(),
        tests: function(opts.function.clone().unwrap()),
    }
}

fn main() {
    let opts: Opts = argh::from_env();
    let machine = mach(opts.arch.clone());
    let m = machine.0;
    let msan = machine.1;

    let testrun = if let Some(path) = opts.file {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let res: DeTestRun = serde_json::from_str(&data).expect("Unable to parse");
        sanity(&res, msan)
    } else {
        testrun_from_args(&opts, msan)
    };

    if opts.search == "exh" {
        let _found_it = |prog: BasicBlock| {
            if equivalence(prog.clone(), &testrun) {
                disassemble(prog);
                true
            } else {
                false
            }
        };
        let _vars: Vec<u16> = vec![3, 4, 5];
        panic!();
    } else if opts.search == "stoc" {
        let convergence = |prog: &BasicBlock| differance(prog, &testrun);
        let vars: Vec<u16> = vec![3, 4, 5];
        let c = constants(opts.constant);
        let prog = stochastic_search(&convergence, msan, &m);
        //let opt = optimize(&convergence, &prog,msan,  &m, &c, &vars);
        println!("finished search");
        disassemble(prog);
    }
}
