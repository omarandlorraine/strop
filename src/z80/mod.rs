//! The Z80 backend for strop
//!
pub mod instruction;
pub use instruction::InstructionZ80;
pub mod emulator;
pub use emulator::EmulatorZ80;
