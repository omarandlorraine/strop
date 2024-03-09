use strop::mos6502::instruction_set::Nmos6502Instruction;
use strop::BruteForceSearch;
use strop::Emulator;
use strop::SearchAlgorithm;

use strop::mos6502::instruction_set::SafeBet;

// A program to discover 6502 programs which behave differently on the two emulators strop
// provides. A program discovered in this way may be a test case exposing a bug in either one of
// the emulators, or perhaps a shortcoming in the static analysis done by strop.

fn main() {
    println!("Strop currently includes three 6502 emulators for running 6502 programs.");
    println!("This program is now searching for programs behaving differently on the three");
    println!("emulators. This is intended to find bugs in the third-party emulators and in");
    println!("the static analysis passes that strop also includes.");

    let mut bruteforce = BruteForceSearch::<Nmos6502Instruction>::new().compatibility(SafeBet);
    for candidate in bruteforce.iter() {
        let mut mos6502 = strop::mos6502::emulators::Mos6502::default();
        let mut nmos6502 = strop::robo6502::emulators::Nmos6502::default();
        let mut cmos6502 = strop::robo6502::emulators::Cmos6502::default();

        mos6502.run(0x8000, &candidate);
        nmos6502.run(0x8000, &candidate);
        cmos6502.run(0x8000, &candidate);

        let m = mos6502.a();
        let nm = nmos6502.a();
        let cm = cmos6502.a();

        if m != nm || m != cm {
            println!("This program results in different accumulator values:");
            candidate.disassemble();
            println!("mos6502:         ${:02x}", m);
            println!("robo6502 (nmos): ${:02x}", nm);
            println!("robo6502 (cmos): ${:02x}", cm);
            println!("*************");
        }
    }
}
