use strop::emulator::Emulator;
use strop::generate::Constraints;
use strop::generate::{McmcSynth, Random};
use strop::mos6502::static_analysis::*;
use strop::mos6502::Emulator6502;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::static_analysis::check_use;

fn mult(sn: &Snippet<Instruction6502>) -> f64 {
    let mut emu: Emulator6502 = Default::default();

    println!("About to run this");
    sn.disassemble();

    emu.run(0x200, 3000, &mut sn.to_bytes().into_iter());

    (emu.get_a().wrapping_sub(45)).into()
}

fn main() {
    let parent = Snippet::<Instruction6502>::default();
    let mut mc = McmcSynth::new(&parent, Constraints::<Instruction6502>::default(), mult);

    // loop until we find at least one candidate program that at least computes the right result
    loop {
        let (score, sn) = mc.next().unwrap();

        println!("one loop iteration, score {}", score);
        sn.disassemble();

        println!();
        println!();

        if score == 0.0 {
            break;
        }
    }
    println!("afterloop");
}
