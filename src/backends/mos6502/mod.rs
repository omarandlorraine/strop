//! A strop backend supporting miscellaneous MOS6502 variants
mod instruction_set;
mod static_analysis;
pub use instruction_set::Instruction;
pub mod zpcall;
pub use static_analysis::do_not_underflow;
pub use static_analysis::do_not_overflow;

#[cfg(test)]
mod test;
