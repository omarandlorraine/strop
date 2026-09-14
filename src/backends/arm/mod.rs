//! Arm backend for strop
//!
//! supporting 32-bit ARM processors beyond ARMv4T

pub enum ArmVariant {
    Armv4,
    Armv5,
    Armv6,
    Armv7,
    Armv8,
}

mod aapcs32;
mod instruction;

pub(crate) trait Instruction: crate::Instruction{}
