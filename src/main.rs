use strop::emulator::Emulator;
use strop::generate::Constraints;
use strop::generate::CorrectPrograms;

use strop::instruction::Instruction;
use strop::mos6502::Emulator6502;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;

use rand::random;

fn set45(sn: &Snippet<Instruction6502>) -> f64 {
    // Checks if the snippet loads the number 45 into the accumulator
    let mut emu: Emulator6502 = Default::default();
    let mut distance: f64 = 0.0;

    // Set the accumulator to a random number (not 45), so we can check that the accumulator gets
    // set to 45 after the putative program has run.
    emu.set_a(random());
    while emu.get_a() == 45 {
        emu.set_a(random());
    }

    emu.run(0x200, 3000, &mut sn.to_bytes().into_iter());

    distance += f64::from(emu.get_a().wrapping_sub(45));
    distance
}

fn main() {
    let constraint = Constraints::<Instruction6502>::new(vec![Instruction6502::perm_bb]);
    let _parent = Snippet::<Instruction6502>::default();
    let mut cp = CorrectPrograms::new(constraint, set45);

    let prog = cp.next().unwrap().1;
    prog.disassemble();
}
