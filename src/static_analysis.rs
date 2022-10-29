use crate::instruction::Instruction;
use crate::snippets::Snippet;

/// VarState keeps track of the state of a Use Before Initialization lint.
#[derive(PartialEq)]
pub enum VarState {
    /// The variable has neither been used nor initialized
    Nothing,
    /// The variable has been read from
    Read,
    /// The variable has been written to
    Written,
    /// The variable has been read and then written to, as would be the case for a RMW operation
    ReadThenWritten,
    /// The variable has been written to and then read from
    WrittenThenRead,
}

impl VarState {
    /// the iterator finds an instruction that reads the variable
    pub fn r(self) -> VarState {
        use crate::static_analysis::VarState::*;
        match self {
            Nothing => Read,
            Written => WrittenThenRead,
            x => x,
        }
    }

    /// the iterator finds an instruction that writes the variable
    pub fn w(self) -> VarState {
        use crate::static_analysis::VarState::*;
        match self {
            Nothing => Written,
            Read => ReadThenWritten,
            x => x,
        }
    }
}

pub fn check_use<I: Instruction + std::fmt::Display + Copy>(
    snippet: &Snippet<I>,
    lint: fn(VarState, &I) -> VarState,
) -> VarState {
    //! Check that the snippet does not use a register (or flag, or variable, or whatever) without first initializing it.
    snippet.vec().iter().fold(VarState::Nothing, lint)
}
