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
    fn to_bytes(&self) -> Vec<u8> {
        self.instructions.iter().map(|insn| insn.as_bytes()).flatten().collect()
    }
}
