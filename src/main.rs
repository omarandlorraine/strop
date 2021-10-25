use std::process;

extern crate argh;
use argh::FromArgs;

mod machine;
mod search;

use crate::machine::{mos6502, mos65c02};
use crate::machine::motorola6800;
use crate::machine::{i8080, i8085, z80, iz80};
use crate::machine::{pic12, pic14, pic16};

use crate::machine::Instruction;
use crate::machine::State;
use crate::machine::{set_a, get_a, set_b, get_b, set_x, set_y, get_x, get_y};

use crate::search::Schema;
use crate::search::equivalence;
use crate::search::exhaustive_search;

struct MOpt {
	name: &'static str, func: fn() -> Vec<Instruction>, help: &'static str
}

struct VOpt {
	name: &'static str, set: fn(&mut State, i8), get: fn(&State) -> Option<i8>, help: &'static str
}

const M_OPTS: [MOpt; 10] = [
	MOpt {name: "i8080",        func: i8080,        help: "Intel 8080"},
	MOpt {name: "i8085",        func: i8085,        help: "Intel 8085"},
	MOpt {name: "iz80",         func: iz80,         help: "for compatibility with both z80 and i8080"},
	MOpt {name: "mos6502",      func: mos6502,      help: "generic 6502"},
	MOpt {name: "mos65c02",     func: mos65c02,     help: "CMOS 6502, including new instructions like phx and stz"},
	MOpt {name: "motorola6800", func: motorola6800, help: "Motorola 6800"},
	MOpt {name: "pic12",        func: pic12,        help: "PIC12"},
	MOpt {name: "pic14",        func: pic14,        help: "PIC14"},
	MOpt {name: "pic16",        func: pic16,        help: "PIC16"},
	MOpt {name: "z80",          func: z80,          help: "Zilog Z80"},
];

const V_OPTS: [VOpt; 4] = [
	VOpt {name: "a", set: set_a, get: get_a, help: "The Accumulator"},
	VOpt {name: "b", set: set_b, get: get_b, help: "The B register"},
	VOpt {name: "x", set: set_x, get: get_x, help: "The X register"},
	VOpt {name: "y", set: set_y, get: get_y, help: "The Y register"},
];

#[derive(FromArgs, PartialEq, Debug)]
/// Specify the machine you want to generate code for.
struct Opts {
	#[argh(option)]
	/// the name of the architecture.
	arch: String,

	#[argh(option)]
	/// the function to compute
	function: String,

	#[argh(option)]
	/// what kind of search to perform
	search: String,

	#[argh(option)]
	/// live_in variables
	live_in: Vec<String>,

	#[argh(option)]
	/// live_out variables
	live_out: Vec<String>,

    #[argh(option)]
    /// constants
    constant: Vec<i8>,
}

fn mach(m: String) -> Vec<Instruction> {
	for m_opt in &M_OPTS {
		if m_opt.name == m {
			return (m_opt.func)();
		}
	}
	println!("You didn't pick a valid arch, so here's the ones I know:");
	for m_opt in &M_OPTS {
		println!("\t{}  {}", format!("{:>12}", m_opt.name), m_opt.help);
	}
	process::exit(1);
}

fn function(m: String) -> Vec<(Vec<i8>, Vec<i8>)> {
	// TODO: test_cases does not need to be mutable..
	let mut test_cases = Vec::new();
	if m == "id" {
		for n in -128..=127 {
			test_cases.push((vec![n], vec![n]));
		}
		return test_cases;
	}
	if m[0..4] == "mult".to_string() {

		let arg = m[4..].to_string();
		let a = arg.parse::<i8>();

		if let Some(f) = a.ok() {
			for n in -128_i8..=127 {
				if let Some(res) = n.checked_mul(f) {
					test_cases.push((vec![n], vec![res]));
				}
			}
			return test_cases;
		} else {
			println!("Can't multiply by {}", arg);
		}
	}
	println!("I don't understand what you mean by the argument {}", m);
	process::exit(1);
}

fn parse_live_in<'a>(arg: String) -> Box<dyn for<'r> Fn(&'r mut State, i8) + 'a > {
	for v_opt in &V_OPTS {
		if v_opt.name == arg {
			return Box::new(v_opt.set);
		}
	}
	println!("I don't understand \"{}\" as a live-in value; here are the ones I know:", arg);
	for v_opt in &V_OPTS {
		println!("\t{}  {}", format!("{:>12}", v_opt.name), v_opt.help);
	}
	process::exit(1);
}

fn parse_live_out<'a>(arg: String) -> Box<dyn for<'r> Fn(&'r State) -> Option<i8> + 'a > {
	for v_opt in &V_OPTS {
		if v_opt.name == arg {
			return Box::new(v_opt.get);
		}
	}
	println!("You didn't pick a valid live-out value, so here's the ones I know:");
	for v_opt in &V_OPTS {
		println!("\t{}  {}", format!("{:>12}", v_opt.name), v_opt.help);
	}
	process::exit(1);
}

fn disassemble(prog: &Vec<Instruction>) {
	for p in prog {
		println!("{}", p);
	}
}

fn constants(c: Vec<i8>) -> Vec<i8> {
    let mut v = Vec::<i8>::new();
    for i in 0..16 {
        v.push(i);
        v.push(0-i);
    }
    for i in 0..7 {
        v.push(2i8.pow(i));
    }
    c.into_iter().chain(v).collect()
}

fn main() {
	let opts: Opts = argh::from_env();
	let schema =  Schema::new(
		opts.live_in.into_iter().map(|arg| parse_live_in(arg)).collect(),
		opts.live_out.into_iter().map(|arg| parse_live_out(arg)).collect());

	let test_cases = function(opts.function);

	if opts.search == "exh" {
		let found_it = |prog: &Vec<Instruction>| {
			if equivalence(prog, &schema, &test_cases){
				disassemble(prog);
				true
			} else {
				false
			}
		};
		exhaustive_search(&found_it, mach(opts.arch), constants(opts.constant));
	}
}
