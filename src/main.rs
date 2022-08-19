extern crate clap;
use crate::machine::mos6502::Mos6502;
use crate::machine::Instruction;
use crate::search::stochastic_search;
use crate::search::BasicBlock;
use rand::random;

pub mod machine;
pub mod search;

use crate::machine::mos6502::Instruction6502;

fn function(bb: &BasicBlock<Instruction6502>) -> f64 {
    // Leave registers A and X having the same value as eachother,
    // and Y having the value 70 (decimal).
    let mut s: Mos6502 = Default::default();
    let mut error_accumulate: f64 = 0.0;
    for _ in 0..5000 {
        s.a = Some(random());
        s.x = Some(random());
        s.y = Some(random());

        for i in &bb.instructions {
            i.operate(&mut s);
        }

        error_accumulate += ((s.a.unwrap_or(255) as f64) - (s.x.unwrap_or(0) as f64)).abs();
        error_accumulate += (s.y.unwrap_or(255) as f64 - 70.0).abs();
    }
    println!("{}", error_accumulate);
    error_accumulate
}

fn main() {
    let bb = stochastic_search::<Instruction6502>(function);

    for insn in bb.instructions {
        println!("{:?}", insn);
    }
}
