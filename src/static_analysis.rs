use crate::instruction::Instruction;
use crate::snippets::Snippet;

pub fn check_use<I: Instruction + std::fmt::Display>(snippet: Snippet<I>, sets: fn(&I) -> bool, requires: fn(&I) -> bool) -> Result<(), usize> {
    //! Check that the snippet does not use a register (or flag, or variable, or whatever) without first initializing it.
    for i in snippet.vec().iter().enumerate() {
        if sets(&i.1) {
            return Ok(());
        };
        if requires(&i.1) {
            return Err(i.0);
        };
    }
    Ok(())
}

