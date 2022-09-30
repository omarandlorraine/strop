use crate::instruction::Instruction;
use crate::snippets::Snippet;

pub enum VarState {
    Nothing,
    Initialized,
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
