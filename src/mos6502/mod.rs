//! The 6502 backend for strop
//!
pub mod instruction;
pub use instruction::Instruction6502;
pub mod emulator;
pub use emulator::Emulator6502;
pub mod data;
pub mod static_analysis;
