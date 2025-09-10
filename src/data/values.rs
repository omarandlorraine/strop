//! Module containing miscellaneous functions for testing callables
use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::UnsupportedArgumentType;

use rand;

/// Returns a few representative values for a given type
///
/// Useful for fuzz testing a Callable
pub trait ReturnValues: std::cmp::PartialEq + Copy + std::fmt::Debug {
    /// Returns the difference between A and B
    fn error(self, other: Self) -> f64;

    #[cfg(feature = "mips")]
    /// Puts the arguments into the argument registers of the MIPS emulator
    fn mips_return_value_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn>;

    #[cfg(feature = "mips")]
    /// Gets the arguments from the value registers of the MIPS emulator
    fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType>;

}

impl ReturnValues for bool {
    fn error(self, other: bool) -> f64 {
        if self == other { 1.0 } else { 0.0 }
    }
}

impl ReturnValues for u8 {
    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl ReturnValues for i8 {
    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl ReturnValues for i16 {
    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl ReturnValues for u16 {
    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
}

impl ReturnValues for f32 {
    fn error(self, other: Self) -> f64 {
        (self - other).abs().into()
    }
}

impl<A: ReturnValues + Copy, B: ReturnValues + Copy> ReturnValues for (A, B) {
    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1)
    }
}

impl<A: ReturnValues + Copy, B: ReturnValues + Copy, C: ReturnValues + Copy> ReturnValues for (A, B, C) {
    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1) + self.2.error(other.2)
    }
}
