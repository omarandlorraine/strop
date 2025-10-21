//! A strop backend targetting very basic MIPS processors

mod bus;
mod instruction_set;
pub use instruction_set::Instruction;
pub mod o32;

#[cfg(test)]
mod test;
