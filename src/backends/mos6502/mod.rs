//! A strop backend supporting miscellaneous MOS6502 variants
mod instruction_set;
pub use instruction_set::Instruction;
pub mod zpcall;

#[cfg(test)]
mod test;
