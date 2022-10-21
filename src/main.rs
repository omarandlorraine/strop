use strop::generate::{McmcSynth, Random};
use strop::mos6502::static_analysis::*;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::static_analysis::check_use;
use strop::mos6502::Emulator6502;
use strop::emulator::Emulator;
use strop::generate::Constraints;

fn mult(sn: &Snippet<Instruction6502>) -> f64 {
    let mut emu: Emulator6502 = Default::default();

    emu.run(0x200, 3000, &mut sn.to_bytes().into_iter());


    println!();
    println!();
    println!();
    sn.disassemble();

    return (emu.get_a().wrapping_sub(45)).into();
}

fn main() {
    // generate an initial population
    let mut population: Vec<_> = Random::<Instruction6502>::new(0x200, 20, mult)
        // Remove all flow control instructions; we'll end up with a linear program without loops,
        // subroutine calls, jumps, returns, etc.
        .map(|(score, sn)| (score, sn.make_bb()))
        // by static analysis, remove programs which use a register or flag without initializing it
        // first.
        .filter(|(_score, sn)| check_use(sn, check_use_c))
        .filter(|(_score, sn)| check_use(sn, check_use_x))
        .filter(|(_score, sn)| check_use(sn, check_use_y))
        // start an initial population of a particular size
        .take(1000)
        .collect();

    // loop until we find at least one candidate program that at least computes the right result
    while population.iter().any(|(score, _sn)| *score == 0.0) {
        let mut next_generation: Vec<_> = population
            .iter()
            .take(100)
            .flat_map(|p| McmcSynth::new(&p.1, Constraints::<Instruction6502>::default(), mult).take(50))
            .map(|(score, p)| (score, p.make_bb()))
            .collect();

        next_generation.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"));

        next_generation.truncate(100);

        population = next_generation;
    }
}
