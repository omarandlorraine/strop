//! Implements searches for functions complying with the AAPCS32 calling convention, as used by
//! modern (EABI) linux systems and others.

use crate::armv4t::Insn;
use crate::BruteforceSearch;
use crate::StaticAnalysis;

/*
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
*/

/// The AAPCS32-compliant function
#[derive(Debug)]
pub struct Function<Params, RetVal> {
    seq: crate::armv4t::Subroutine,
    params: std::marker::PhantomData<Params>,
    retval: std::marker::PhantomData<RetVal>,
}

impl<Params, RetVal> crate::Disassemble for Function <Params, RetVal>{
    fn dasm(&self) {
        self.seq.dasm()
    }
}

impl<Params, RetVal> crate::Goto<Insn> for Function <Params, RetVal>{
    fn goto(&mut self, t: &[Insn]) {
        self.seq.goto(t);
    }
}

impl<Params, RetVal> BruteforceSearch<Insn> for Function<Params, RetVal> {
    fn analyze_this(&self) -> Option<StaticAnalysis<Insn>> {
        // TODO: dataflow analysis could go here.
        None
    }
    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}
