use strop::emulator::Emulator;
use strop::generate::Constraints;
use strop::generate::CorrectPrograms;

use strop::instruction::Instruction;
use strop::mos6502::Emulator6502;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::static_analysis::check_use;
use strop::static_analysis::VarState;

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

fn mul10(sn: &Snippet<Instruction6502>) -> f64 {
    use strop::mos6502::static_analysis::check_use_a;
    // Checks if the snippet loads the number 45 into the accumulator
    let mut emu: Emulator6502 = Default::default();
    let mut distance: f64 = 0.0;

    if check_use(&sn, check_use_a) != VarState::ReadThenWritten {
        return f64::MAX;
    }

    // Set the accumulator to a random number, and if the number times 10 still fits in a byte,
    // check that the program multiplies the number correctly.
    for i in 0u8..255 {
        let before: u8 = i;
        emu.set_a(before);

        if let Some(result) = before.checked_mul(10) {
            emu.run(0x200, 3000, &mut sn.to_bytes().into_iter());

            distance += f64::from((emu.get_a() ^ result).count_ones());

        }
    }

    println!();
    println!();
    println!();
    sn.disassemble();
    println!("{:.}", distance);
    distance
}

fn main() {
    let constraint = Constraints::<Instruction6502>::new(vec![Instruction6502::perm_bb]);
    let _parent = Snippet::<Instruction6502>::default();
    let mut cp = CorrectPrograms::new(constraint, mul10);

    let prog = cp.next().unwrap().1;
    prog.disassemble();
}
