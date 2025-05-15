use crate::mips::Insn;

/// Represents a MIPS subroutine
pub type Subroutine = crate::Subroutine<Insn, crate::Sequence<Insn>>;

#[cfg(test)]
mod test {
    #[test]
    fn all_two_instruction_subroutines() {
        // To make sure that all the generated instructions can at least be executed by the
        // emulator, I'm going to generate all subroutines having a length of two (the last
        // instruction is of course always going to be `jr $ra`) and run them all in the emulator.

        use crate::Step;

        use crate::Disassemble;
        use crate::Encode;

        let mut sub = super::Subroutine::first();
        sub.dasm();
        while sub.len() <= 8 {
            println!("trying to run this subroutine:");
            sub.dasm();
            crate::mips::emu::call_raw(&sub).unwrap();
            sub.next().unwrap();
        }
    }
}
