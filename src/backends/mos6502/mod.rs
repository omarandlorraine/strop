//! A strop backend supporting miscellaneous MOS6502 variants
mod instruction_set;
pub use instruction_set::Instruction;

#[cfg(test)]
mod test;
