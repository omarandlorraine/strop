use strop::mos6502::static_analysis::*;
use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::static_analysis::check_use;
use strop::generate::Random;

fn main() {
    /*
    loop {
        let sn = Snippet::<Instruction6502>::new();
        if !check_use(&sn, check_use_c) {
            sn.disassemble();
            break;
        }
    }
    */

    // generate an initial population
    let population = Random::<Instruction6502>::new(0x200, 20)
        // Remove all flow control instructions; we'll end up with a linear program without loops,
        // subroutine calls, jumps, returns, etc.
        .map(|p| p.make_bb())

        // by static analysis, remove programs which use a register or flag without initializing it
        // first.
        .filter(|p| check_use(p, check_use_c))
        .filter(|p| check_use(p, check_use_x))
        .filter(|p| check_use(p, check_use_y))

        // start an initial population of a particular size
        .take(1000);

    
}
