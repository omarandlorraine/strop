/// the Snippets module
///
/// A snippet is a (possibly short) list of assembly instructions, all intended to be executed one
/// after another.
///
/// In earlier versions of Strop, this was called BasicBlock, but as we're lifting some
/// restrictions this no longer needs to be a basic block.
///
///
use crate::instruction::Instruction;
use crate::randomly;
use rand::thread_rng;
use rand::Rng;

#[derive(Clone, Debug)]
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

impl<I: Instruction + std::fmt::Display + Copy> Snippet<I> {
    pub fn new() -> Self {
        Self {
            org: 0x0200,
            instructions: (1..10).map(|_| I::new()).collect(),
        }
    }

    pub fn new_with_org_and_length(org: usize, max_length: usize) -> Self {
        let i = thread_rng().gen_range(0..max_length);

        Self {
            org,
            instructions: (1..i).map(|_| I::new()).collect(),
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

    pub fn mutate(&mut self) {
        if self.instructions.is_empty() {
            // The only mutation we can do here is to insert random instructions
            self.instructions.push(I::new());
            return;
        }
        let offset = thread_rng().gen_range(0..self.instructions.len());
        randomly!(
        {
            // remove a random instruction
            self.instructions.remove(offset);
        }
        {
            // insert a randomly generated instruction somewhere in the program
            self.instructions.insert(offset, I::new());
        }
        {
            // replace one instruction with a randomly generated one
            self.instructions[offset] = I::new();
        }
        {
            // pick an instruction at random, and modify its operand
            self.instructions[offset].mutate_operand();
        }
        {
            // pick an instruction at random, and modify its opcode
            self.instructions[offset].mutate_opcode();
        }
        {
            // pick two instructions at random, and swap them over
            let offs2 = thread_rng().gen_range(0..self.instructions.len());
            self.instructions.swap(offset, offs2);
        }
        );

        todo!()
        // the miscellaneous mutations should go here
    }
}
