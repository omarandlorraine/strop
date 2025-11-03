//! Strop's backends go in here.

#[cfg(feature = "armv4t")]
pub mod armv4t;

#[cfg(feature = "m6502")]
pub mod mos6502;

#[cfg(feature = "mips")]
pub mod mips;

#[cfg(feature = "sm83")]
pub mod sm83;

#[cfg(any(feature = "sm83", feature = "z80"))]
pub(crate) mod x80;

#[cfg(feature = "z80")]
pub mod z80;
