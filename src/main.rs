use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::z80::InstructionZ80;
use strop::static_analysis::check_use;
use strop::mos6502::static_analysis::*;

fn main() {
    loop {
        let sn = Snippet::<Instruction6502>::new();
        if !check_use(&sn, check_use_c) {
            sn.disassemble();
            break;
        }
    }
}
