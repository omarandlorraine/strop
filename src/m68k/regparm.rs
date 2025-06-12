//! Implements searches for functions complying with the regparm calling convention, roughly what
//! GCC-M68K seems to do.

use crate::m68k::emu::Emulator;
use crate::m68k::isa::Insn;
use crate::BruteforceSearch;
use crate::Callable;
use crate::StaticAnalysis;

pub trait Parameters {}
