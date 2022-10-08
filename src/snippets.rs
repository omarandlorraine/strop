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

#[derive(Debug)]
pub struct Snippet<I> {
    /// The list of instructions in the snippet
    pub instructions: Vec<I>,

    /// Start address
    org: usize,
}

impl<I: Instruction> Default for Snippet<I> {
    fn default() -> Self {
        Snippet {
            org: 0x0200,
            instructions: vec![],
        }
    }
}

impl<I: Instruction + std::fmt::Display> Snippet<I> {
    pub fn new() -> Self {
        Self {
            org: 0x0200,
            instructions: (1..10).map(|_| I::new()).collect(),
        }
    }

    pub fn vec(&self) -> Vec<I> {
        self.instructions.clone()
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.instructions
            .iter()
            .flat_map(|insn| insn.as_bytes())
            .collect()
    }

    pub fn disassemble(&self) {
        let mut address = self.org;
        for i in &self.instructions {
            println!("  ${:04x}  {}", address, i);
            address += i.to_bytes().len();
        }
    }

    /// Makes sure that the snippet is a basic block. (i.e., if you call this method, it will
    /// mutate the snippet in such a way, that it will not contain any branches, jumps, subroutine
    /// calls, returns, or other flow control operations).
    pub fn make_bb(&mut self) {
        for insn in &mut self.instructions {
            while !insn.perm_bb() {
                *insn = I::new();
            }
        }
    }

    pub fn retain(&mut self, filterfn: fn(&I) -> bool) {
        self.instructions.retain(filterfn);
    }
}
