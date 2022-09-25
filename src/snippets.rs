/// the Snippets module
///
/// A snippet is a (possible short) list of assembly instructions, all intended to be executed one
/// after another.
///
/// In earlier versions of Strop, this was called BasicBlock, but as we're lifting some
/// restrictions this no longer needs to be a basic block.
///
///
use crate::instruction::Instruction;

pub struct Snippet<I> {
    /// The list of instructions in the snippet
    pub instructions: Vec<I>,
}

impl<I: Instruction> Default for Snippet<I> {
    fn default() -> Self {
        Snippet {
            instructions: vec![],
        }
    }
}

impl<I: Instruction> Snippet<I> {
    fn new(&self) -> Self {
        Self {
            instructions: (1..10).map(|_| I::new()).collect()
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.instructions
            .iter()
            .map(|insn| insn.as_bytes())
            .flatten()
            .collect()
    }

    fn check_use(&self, sets: fn(&I) -> bool, requires: fn(&I) -> bool) {
        /// Check that the snippet does not use a register without first initializing it.
        for i in self.instructions {
            if sets(i) { return true };
            if requires(i) { return false };
        }
        return true;
    }

    pub fn disassemble(&self) {
        // todo: Can this use yaxpeax-dis somehow instead?
        let mut address = self.org;
        for i in &self.instructions {
            println!("  ${:04x}  {}", address, i);
            address += i.to_bytes().len();
        }
    }
}