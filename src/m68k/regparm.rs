//! Implements searches for functions complying with the regparm calling convention, roughly what
//! GCC-M68K seems to do.

use crate::m68k::emu::Emulator;
use crate::m68k::isa::Insn;
use crate::BruteforceSearch;
use crate::Callable;
use crate::StaticAnalysis;
use crate::Sequence;
use crate::RunResult;

pub trait Parameters {
    fn install(&self, emu: &mut Emulator);
}

impl Parameters for u32 {
    fn install(&self, emu: &mut Emulator) {
        emu.set_d0(*self);
    }
}

pub trait ReturnValue {
    fn extract(emu: &Emulator) -> Self;
}

impl ReturnValue for u32 {
    fn extract(emu: &Emulator) -> Self {
        emu.get_d0()
    }
}

#[derive(Clone, Default)]
pub struct Regparm {
    seq: Sequence<Insn>
}

impl<Params: Parameters, RetVal: ReturnValue> Callable<Params, RetVal> for Regparm {
    fn call(&self, parameters: Params) -> RunResult<RetVal> {
        let mut emu = Emulator::new();
        parameters.install(&mut emu);
        emu.call_subroutine(&self.seq)?;
        Ok(RetVal::extract(&emu))
    }
}

impl BruteforceSearch<Insn> for Regparm {
    fn analyze_this(&self) -> Result<(), StaticAnalysis<Insn>> {
        crate::subroutine::make_return(&self.seq)?;
        Ok(())
    }

    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.seq
    }
}
