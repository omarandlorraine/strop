use strop::mos6502::Instruction6502;
use strop::snippets::Snippet;
use strop::z80::InstructionZ80;

fn main() {
    loop {
        let sn = Snippet::<Instruction6502>::new();
        if sn.check_use(Instruction6502::sets_x, Instruction6502::reads_x) {
            sn.disassemble();
            break;
        }
        let sn = Snippet::<InstructionZ80>::new();
        sn.disassemble();
    }
}
