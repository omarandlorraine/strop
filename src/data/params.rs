//! Module containing miscellaneous functions for testing callables
use crate::Callable;
use crate::Sequence;
use crate::StaticAnalysis;
use crate::UnsupportedArgumentType;
use rand;

macro_rules! impl_mips_scalar {
    () => {
        #[cfg(feature = "mips")]
        fn to_mips_cpu(
            &self,
            cpu: &mut trapezoid_core::cpu::Cpu,
        ) -> Result<crate::mips::emu::Cpu, UnsupportedArgumentType> {
            crate::mips::dataflow::put1(cpu, *self);
            Ok(())
        }
        #[cfg(feature = "mips")]
        fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType> {
            Ok(crate::mips::dataflow::get1(cpu))
        }
        #[cfg(feature = "mips")]
        fn mips_parameter_dataflow(
            seq: &Sequence<crate::mips::Insn>,
        ) -> StaticAnalysis<crate::mips::Insn> {
            crate::mips::dataflow::number_of_arguments(seq, 1)
        }
        #[cfg(feature = "mips")]
        fn mips_return_value_dataflow(
            seq: &Sequence<crate::mips::Insn>,
        ) -> StaticAnalysis<crate::mips::Insn> {
            crate::mips::dataflow::number_of_return_values(seq, 1)
        }
    };
}

macro_rules! impl_mips_dyadic {
    () => {
        #[cfg(feature = "mips")]
        fn mips_put(
            &self,
            cpu: &mut trapezoid_core::cpu::Cpu,
        ) -> Result<(), UnsupportedArgumentType> {
            crate::mips::dataflow::put1(cpu, *self);
            Ok(())
        }
        #[cfg(feature = "mips")]
        fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType> {
            Ok(crate::mips::dataflow::get1(cpu))
        }
        #[cfg(feature = "mips")]
        fn mips_parameter_dataflow(
            seq: &Sequence<crate::mips::Insn>,
        ) -> StaticAnalysis<crate::mips::Insn> {
            crate::mips::dataflow::number_of_arguments(seq, 2)
        }
        #[cfg(feature = "mips")]
        fn mips_return_value_dataflow(
            seq: &Sequence<crate::mips::Insn>,
        ) -> StaticAnalysis<crate::mips::Insn> {
            crate::mips::dataflow::number_of_return_values(seq, 2)
        }
    };
}

pub trait Parameters: std::cmp::PartialEq + Copy + std::fmt::Debug {
    /// Returns a few representative values
    fn vals() -> Vec<Self>
    where
        Self: std::marker::Sized;

    /// Returns a random value
    fn rand() -> Self;

    /// Returns the difference between A and B
    fn error(self, other: Self) -> f64;

    #[cfg(feature = "mips")]
    /// Returns a MIPS CPU emulator with the arguments already put in place
    fn to_mips_cpu(&self, cpu: &mut trapezoid_core::cpu::Cpu) -> Result<crate::mips::emu::Cpu, UnsupportedArgumentType>;

    #[cfg(feature = "mips")]
    /// Puts the arguments into the argument registers of the MIPS emulator
    fn mips_put(&self, cpu: &mut trapezoid_core::cpu::Cpu) -> Result<(), UnsupportedArgumentType>;

    #[cfg(feature = "mips")]
    /// Puts the arguments into the argument registers of the MIPS emulator
    fn mips_parameter_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn>;

    #[cfg(feature = "mips")]
    /// Puts the arguments into the argument registers of the MIPS emulator
    fn mips_return_value_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn>;

    #[cfg(feature = "mips")]
    /// Gets the arguments from the value registers of the MIPS emulator
    fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType>;
}

impl Parameters for bool {
    fn vals() -> Vec<Self> {
        vec![true, false]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: bool) -> f64 {
        if self == other { 1.0 } else { 0.0 }
    }
    impl_mips_scalar!();
}

impl Parameters for u8 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..8 {
            v.push(1 << i);
            v.push(i);
            v.push(u8::MAX - i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }
    impl_mips_scalar!();
}

impl Parameters for i8 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(i8::MAX - i);
            v.push(i8::MIN + i);
            v.push(-i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }

}

impl Parameters for i16 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(i16::MAX - i);
            v.push(i16::MIN + i);
            v.push(-i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }

}

impl Parameters for u16 {
    fn vals() -> Vec<Self> {
        let mut v = vec![0];
        for i in 0..16 {
            v.push(1 << i);
            v.push(i);
            v.push(u16::MAX - i);
            v.push(u16::MIN + i);
        }
        v
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self ^ other).count_ones() as f64
    }

}

impl Parameters for f32 {
    fn vals() -> Vec<Self> {
        vec![0.0, -1.0, 1.0, -0.5, 0.5]
    }

    fn rand() -> Self {
        rand::random()
    }

    fn error(self, other: Self) -> f64 {
        (self - other).abs().into()
    }

    #[cfg(feature = "mips")]
    fn mips_put(&self, cpu: &mut trapezoid_core::cpu::Cpu) -> Result<(), UnsupportedArgumentType> {
        Err(UnsupportedArgumentType::FloatingPointNotSupported)
    }
    #[cfg(feature = "mips")]
    fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType> {
        Err(UnsupportedArgumentType::FloatingPointNotSupported)
    }
    #[cfg(feature = "mips")]
    fn mips_parameter_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn> {
        crate::mips::dataflow::number_of_arguments(seq, 1)
    }
    #[cfg(feature = "mips")]
    fn mips_return_value_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn> {
        crate::mips::dataflow::number_of_arguments(seq, 1)
    }
}

impl<A: Parameters + Copy, B: Parameters + Copy> Parameters for (A, B) {
    fn vals() -> Vec<Self> {
        let mut v = vec![];
        for a in A::vals() {
            for b in B::vals() {
                v.push((a, b));
            }
        }
        v
    }

    fn rand() -> Self {
        (A::rand(), B::rand())
    }

    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1)
    }
    #[cfg(feature = "mips")]
    fn mips_put(&self, cpu: &mut trapezoid_core::cpu::Cpu) -> Result<(), UnsupportedArgumentType> {
        crate::mips::dataflow::put2(cpu, self.0 as u32, self.1 as u32)?;
    }
    #[cfg(feature = "mips")]
    fn mips_get(cpu: &trapezoid_core::cpu::Cpu) -> Result<Self, UnsupportedArgumentType> {
        Ok((cpu.get_v0(), cpu.get_v1()))
    }
    #[cfg(feature = "mips")]
    fn mips_parameter_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn> {
        crate::mips::dataflow::number_of_arguments(seq, 2)
    }
    #[cfg(feature = "mips")]
    fn mips_return_value_dataflow(
        seq: &Sequence<crate::mips::Insn>,
    ) -> StaticAnalysis<crate::mips::Insn> {
        crate::mips::dataflow::number_of_arguments(seq, 2)
    }
}

impl<A: Parameters + Copy, B: Parameters + Copy, C: Parameters + Copy> Parameters for (A, B, C) {
    fn vals() -> Vec<Self> {
        let mut v = vec![];
        for a in A::vals() {
            for b in B::vals() {
                for c in C::vals() {
                    v.push((a, b, c));
                }
            }
        }
        v
    }

    fn rand() -> Self {
        (A::rand(), B::rand(), C::rand())
    }

    fn error(self, other: Self) -> f64 {
        self.0.error(other.0) + self.1.error(other.1) + self.2.error(other.2)
    }

}
