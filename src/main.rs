use strop::emulator::Emulator;
use strop::generate::Constraints;
use strop::generate::{McmcSynth, Random};
use strop::mos6502::static_analysis::*;
use strop::mos6502::Emulator6502;
use strop::instruction::Instruction;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::static_analysis::check_use;
use rand::random;

fn mult(sn: &Snippet<Instruction6502>) -> f64 {
    let mut emu: Emulator6502 = Default::default();
    let mut distance: f64 = 0.0;

    // For reasons, doing this loop and setting the accumulator to a random number each time, it
    // decreases the chance of an erroneous program being generated, such as:
    //     adc #$2d
    // We need to apply static analysis instead, and then get rid of the loop. Should yield an
    // almost hundredfold speedup.
    for _ in 0..100 {
        emu.set_a(random());

        emu.run(0x200, 3000, &mut sn.to_bytes().into_iter());

        distance += f64::from((emu.get_a().wrapping_sub(45)));
    }
    distance
}

fn main() {
    let constraint = Constraints::<Instruction6502>::new(vec![Instruction6502::perm_bb]);
    let parent = Snippet::<Instruction6502>::default();
    let mut mc = McmcSynth::new(&parent, constraint, mult);

    // loop until we find at least one candidate program that at least computes the right result
    let mut correct = Snippet::<Instruction6502>::default();
    loop {
        let (score, sn) = mc.next().unwrap();

        if score == 0.0 {
            correct = sn;
            break;
        }
    }
    correct.disassemble();
    println!("afterloop");
}
