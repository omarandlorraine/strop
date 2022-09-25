use strop::instruction::mos6502::Instruction6502;
use strop::snippets::Snippet;

fn main() {
    loop {
        let sn = Snippet::<Instruction6502>::new();
        if sn.check_use(Instruction6502::sets_x, Instruction6502::reads_x) {
            sn.disassemble();
            break;
        }
    }
}
