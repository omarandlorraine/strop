use strop::emulator::Emulator;
use strop::generate::Constraints;
use strop::generate::{McmcSynth, Random};
use strop::mos6502::static_analysis::*;
use strop::mos6502::Emulator6502;
use strop::instruction::Instruction;
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
