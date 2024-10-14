//! Implements searches for functions complying with the AAPCS32 calling convention, as used by
//! modern (EABI) linux systems and others.

use crate::armv4t::isa::decode::Register;
use crate::armv4t::Insn;
use crate::Sequence;

fn callee_saved(r: &Register) -> bool {
    match r {
        Register::R0 => false,
        Register::R1 => false,
        Register::R2 => false,
        Register::R3 => false,
        Register::R4 => true,
        Register::R5 => true,
        Register::R6 => true,
        Register::R7 => true,
        Register::R8 => true,
        Register::R9 => true,
        Register::R10 => true,
        Register::R11 => true,
        Register::R12 => false,
        Register::Sp => false,
        Register::Lr => false,
        Register::Pc => false,
    }
}

fn prologue(r: &[Register]) -> Vec<Insn> {
    if r.is_empty() {
        vec![]
    } else {
        vec![Insn::push(r)]
    }
}

fn epilogue(r: &[Register]) -> Vec<Insn> {
    if r.is_empty() {
        vec![Insn::bx_lr()]
    } else {
        let mut r = r.to_owned();
        r.push(Register::Pc);
        vec![Insn::pop(&r)]
    }
}

/// The AAPCS32-compliant function
#[derive(Debug)]
pub struct Function(Sequence<Insn>);

impl Function {
    /// Builds the function by concatenating the prologue, the body of the subroutine, and the
    /// epilogue. The prologue and epilogue are made to save and restore the callee-saved
    /// registers.
    pub fn build(&self) -> Sequence<Insn> {
        let mut unique_elements = std::collections::HashSet::new();

        let callee_saved_registers: Vec<_> = self
            .0
            .iter()
            .map(|i| i.uses())
            .flat_map(|v| v.into_iter())
            .filter(|item| unique_elements.insert(*item))
            .filter(callee_saved)
            .collect();

        let prologue = prologue(&callee_saved_registers);
        let epilogue = epilogue(&callee_saved_registers);
        vec![&prologue, &self.0, &epilogue].into()
    }
}

impl crate::Disassemble for Function {
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl crate::Goto<Insn> for Function {
    fn goto(&mut self, t: &[Insn]) {
        self.0.goto(t);
    }
}

impl crate::Iterable for Function {
    fn first() -> Self {
        Self(crate::Iterable::first())
    }

    fn step(&mut self) -> bool {
        self.0.step()
    }
}
