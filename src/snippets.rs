/// the Snippets module
///
/// A snippet is a (possibly short) list of assembly instructions, all intended to be executed one
/// after another.
///
/// In earlier versions of Strop, this was called BasicBlock, but as we're lifting some
/// restrictions this no longer needs to be a basic block.
///
use crate::generate::Constraints;
use crate::instruction::Instruction;
use crate::randomly;
use rand::thread_rng;
use rand::Rng;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Snippet<'a, I> {
    /// The list of instructions in the snippet
    pub instructions: Vec<I>,

    /// Start address
    org: usize,

    // need to use `'a` somewhere
    _marker: PhantomData<&'a ()>,
}

impl<'a, I: Instruction> Default for Snippet<'a, I> {
    fn default() -> Self {
        Snippet {
            org: 0x0200,
            instructions: vec![],
            _marker: PhantomData,
        }
    }
}

impl<'a, I: Instruction + std::fmt::Display + Copy> Snippet<'a, I> {
    pub fn new() -> Self {
        Self {
            org: 0x0200,
            instructions: (1..10).map(|_| I::new()).collect(),
            _marker: PhantomData,
        }
    }

    pub fn new_with_org_and_length(org: usize, max_length: usize) -> Self {
        let i = thread_rng().gen_range(0..max_length);

        Self {
            org,
            instructions: (1..i).map(|_| I::new()).collect(),
            _marker: PhantomData,
        }
    }

    pub fn vec(&self) -> Vec<I> {
        self.instructions.clone()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
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
    pub fn make_bb(&self) -> Self {
        let mut copy = self.clone();
        for insn in &mut copy.instructions {
            while !insn.perm_bb() {
                *insn = I::new();
            }
        }
        copy
    }

    pub fn retain(&mut self, filterfn: fn(&I) -> bool) {
        self.instructions.retain(filterfn);
    }

    pub fn mutate(&mut self, constraint: &Constraints<I>) {
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
            if let Some(insn) = constraint.new_instruction() {
                self.instructions.insert(offset, insn);
            }
        }
        {
            // replace one instruction with a randomly generated one
            if let Some(insn) = constraint.new_instruction() {
                self.instructions[offset] = insn;
            }
        }
        {
            // pick an instruction at random, and modify its operand
            for _ in 0..5 {
                let mut insn = self.instructions[offset];
                insn.mutate_operand();
                if constraint.allow(&insn) {
                    self.instructions[offset] = insn;
                    break;
                }
            }
        }
        {
            // pick an instruction at random, and modify its opcode
            for _ in 0..5 {
                let mut insn = self.instructions[offset];
                insn.mutate_opcode();
                if constraint.allow(&insn) {
                    self.instructions[offset] = insn;
                    break;
                }
            }
        }
        {
            // pick two instructions at random, and swap them over
            let offs2 = thread_rng().gen_range(0..self.instructions.len());
            self.instructions.swap(offset, offs2);
        }
        );
    }
}
