use crate::instruction::Instruction;
use crate::snippets::Snippet;

/// VarState keeps track of the state of a Use Before Initialization lint.
#[derive(PartialEq)]
pub enum VarState {
    /// The variable has neither been used nor initialized
    Nothing,
    /// The variable has been initialized
    Initialized,
    /// The variable has neither been used but not initialized. This is normally the state we want
    /// to avoid.
    UseBeforeInit,
}

impl VarState {
    /// the iterator finds a use of the variable
    pub fn used(&self) -> VarState {
        use crate::static_analysis::VarState::*;
        match self {
            Nothing => UseBeforeInit,
            UseBeforeInit => UseBeforeInit,
            Initialized => Initialized,
        }
    }

    /// the iterator finds an initialization of the variable
    pub fn init(&self) -> VarState {
        use crate::static_analysis::VarState::*;
        match self {
            Nothing => UseBeforeInit,
            UseBeforeInit => UseBeforeInit,
            Initialized => Initialized,
        }
    }
}

pub fn check_use<I: Instruction + std::fmt::Display>(
    snippet: Snippet<I>,
    lint: fn(VarState, &I) -> VarState,
) -> bool {
    //! Check that the snippet does not use a register (or flag, or variable, or whatever) without first initializing it.
    snippet.vec().iter().fold(VarState::Nothing, lint) != VarState::UseBeforeInit
}
