use std::fs;
use std::process;

extern crate argh;
use argh::FromArgs;

mod machine;
mod search;
mod test;

use crate::machine::motorola6800;
use crate::machine::{i8080, i8085, iz80, z80};
use crate::machine::{mos6502, mos65c02};
use crate::machine::{pic12, pic14, pic16};

use crate::machine::Instruction;
use crate::machine::State;
use crate::machine::Register;

use crate::search::BasicBlock;
use crate::search::{differance, equivalence};
use crate::search::{exhaustive_search, optimize, stochastic_search};

use crate::test::{sanity, DeParameter, DeTestRun, Parameter, Test, TestRun};

struct MOpt {
    name: &'static str,
    func: fn() -> Vec<Instruction>,
    sanity: fn(&DeParameter) -> Parameter,
    help: &'static str,
}

fn registers8080(regname: &Option<String>) -> Option<Parameter> {
    if let Some(r) = regname {
        match r.as_str() {
            "a" => Some(Parameter {
                name: "a".to_string(),
                address: None,
                cost: None,
                register: Register::A
            }),
            "b" => Some(Parameter {
                name: "b".to_string(),
                address: None,
                cost: None,
                register: Register::B
            }),
            // TODO: The rest of the registers for this architecture
            _ => None,
        }
    } else {
        None
    }
}

fn registers6502(regname: &Option<String>) -> Option<Parameter> {
    if let Some(r) = regname {
        match r.as_str() {
            "a" => Some(Parameter {
                name: "a".to_string(),
                address: None,
                cost: None,
                register: Register::A
            }),
            "x" => Some(Parameter {
                name: "x".to_string(),
                address: None,
                cost: None,
                register: Register::X
            }),
            "y" => Some(Parameter {
                name: "y".to_string(),
                address: None,
                cost: None,
                register: Register::Y
            }),
            _ => None,
        }
    } else {
        None
    }
}

fn registers6800(regname: &Option<String>) -> Option<Parameter> {
    if let Some(r) = regname {
        match r.as_str() {
            "a" => Some(Parameter {
                name: "a".to_string(),
                address: None,
                cost: None,
                register: Register::A
            }),
            "b" => Some(Parameter {
                name: "b".to_string(),
                address: None,
                cost: None,
                register: Register::B
            }),
            // TODO: The rest of the registers for this architecture
            _ => None,
        }
    } else {
        None
    }
}

fn registers_pic(regname: &Option<String>) -> Option<Parameter> {
    if let Some(r) = regname {
        match r.as_str() {
            // just use the stter & getter for the A register
            "w" => Some(Parameter {
                name: "w".to_string(),
                address: None,
                cost: None,
                register: Register::A
            }),
            _ => None,
        }
    } else {
        None
    }
}

fn sanity_i8080(dp: &DeParameter) -> Parameter {
    if let Some(dp) = registers8080(&dp.register) {
        dp
    } else {
        panic!(
            "No such register as {} for the specified architecture.",
            dp.register.as_ref().unwrap()
        );
    }
}

fn sanity_mos6502(dp: &DeParameter) -> Parameter {
    if let Some(dp) = registers6502(&dp.register) {
        dp
    } else {
        panic!(
            "No such register as {} for the specified architecture.",
            dp.register.as_ref().unwrap()
        );
    }
}

fn sanity_6800(dp: &DeParameter) -> Parameter {
    if let Some(dp) = registers6800(&dp.register) {
        dp
    } else {
        panic!(
            "No such register as {} for the specified architecture.",
            dp.register.as_ref().unwrap()
        );
    }
}

fn sanity_pic(dp: &DeParameter) -> Parameter {
    if let Some(dp) = registers_pic(&dp.register) {
        dp
    } else {
        panic!(
            "No such register as {} for the specified architecture.",
            dp.register.as_ref().unwrap()
        );
    }
}

const M_OPTS: [MOpt; 10] = [
    MOpt {
        name: "i8080",
        func: i8080,
        sanity: sanity_i8080,
        help: "Intel 8080",
    },
    MOpt {
        name: "i8085",
        func: i8085,
        sanity: sanity_i8080,
        help: "Intel 8085",
    },
    MOpt {
        name: "iz80",
        func: iz80,
        sanity: sanity_i8080,
        help: "for compatibility with both z80 and i8080",
    },
    MOpt {
        name: "mos6502",
        func: mos6502,
        sanity: sanity_mos6502,
        help: "generic 6502",
    },
    MOpt {
        name: "mos65c02",
        func: mos65c02,
        sanity: sanity_mos6502,
        help: "CMOS 6502, including new instructions like phx and stz",
    },
    MOpt {
        name: "motorola6800",
        func: motorola6800,
        sanity: sanity_6800,
        help: "Motorola 6800",
    },
    MOpt {
        name: "pic12",
        func: pic12,
        sanity: sanity_pic,
        help: "PIC12",
    },
    MOpt {
        name: "pic14",
        func: pic14,
        sanity: sanity_pic,
        help: "PIC14",
    },
    MOpt {
        name: "pic16",
        func: pic16,
        sanity: sanity_pic,
        help: "PIC16",
    },
    MOpt {
        name: "z80",
        func: z80,
        sanity: sanity_i8080, // TODO: needs a different sanity checker.
        help: "Zilog Z80",
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

fn mach(m: String) -> (Vec<Instruction>, fn(&DeParameter) -> Parameter) {
    for m_opt in &M_OPTS {
        if m_opt.name == m {
            return ((m_opt.func)(), m_opt.sanity);
        }
    }
    println!("You didn't pick a valid arch, so here's the ones I know:");
    for m_opt in &M_OPTS {
        println!("\t{}  {}", format!("{:>12}", m_opt.name), m_opt.help);
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

fn testrun_from_args(opts: &Opts, pinj: fn(&DeParameter) -> Parameter) -> TestRun {
    TestRun {
        ins: opts
            .r#in
            .clone()
            .into_iter()
            .map(|reg| pinj(&DeParameter{register: Some(reg.clone()), cost: Some(0.0), address: None, name: Some(reg.clone())}))
            .collect(),
        outs: opts
            .out
            .clone()
            .into_iter()
            .map(|reg| pinj(&DeParameter{register: Some(reg.clone()), cost: Some(0.0), address: None, name: Some(reg.clone())}))
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
        let found_it = |prog: BasicBlock| {
            if equivalence(prog.clone(), &testrun) {
                disassemble(prog);
                true
            } else {
                false
            }
        };
        let vars: Vec<u16> = vec![3, 4, 5];
        exhaustive_search(&found_it, m, constants(opts.constant), vars);
    } else if opts.search == "stoc" {
        let convergence = |prog: &BasicBlock| differance(prog, &testrun);
        let vars: Vec<u16> = vec![3, 4, 5];
        let c = constants(opts.constant);
        let prog = stochastic_search(&convergence, &m, &c, &vars);
        let opt = optimize(&convergence, &prog, &m, &c, &vars);
        disassemble(opt);
    }
}
