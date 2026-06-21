//! A backend for strop for targetting embedded ARM processors.

mod thumb;

mod aapcs32;
pub use aapcs32::Aapcs32;

pub use thumb::ThumbInstruction;

use thumb::{BadInstructionRange, bad_instruction_ranges};
/// A trait implemented by particular ARM variants
pub trait Variant: Sized {
    /// Checks that the thumb instruction is valid for this ARM variant.
    fn check(insn: &ThumbInstruction<Self>) -> Result<(), BadInstructionRange>;

    /*
    /// If `i` doesn't disassemble, then return the next valid encoding
    fn undisassemblable(i: u16) -> Option<u16> {
        // list of ranges of instructions which unarm treats as illegal instructions
        for ill in [
            0xb100..0xb200,
            0xb300..0xb401,
            0xb600..0xb650,
            0xb651..0xb658,
            0xb659..0xb660,
            0xb659..0xb660,
            0xb668..0xb670,
            0xb678..0xba00,
            0xba80..0xbac0,
            0xbb00..0xbc01,
            0xbf00..0xc001,
            0xc100..0xc101,
            0xc200..0xc201,
            0xc300..0xc301,
            0xc400..0xc401,
            0xc500..0xc501,
            0xc600..0xc601,
            0xc700..0xc701,
            0xc800..0xc801,
            0xc900..0xc901,
            0xca00..0xca01,
            0xcb00..0xcb01,
            0xcc00..0xcc01,
            0xcd00..0xcd01,
            0xce00..0xce01,
            0xcf00..0xcf01,
            0xe800..0xffff,
        ] {
            if ill.contains(&i) {
                return Some(ill.end);
            }
        }
        None
    }
    */

    /// Return the human-readable name of this variant
    fn name() -> &'static str;

    /// Return an emulated processor
    fn proc() -> armagnac::core::Processor;
}

/// ArmV6-M
#[derive(Clone)]
pub(crate) struct Armv6m;
impl Variant for Armv6m {
    fn check(insn: &ThumbInstruction<Self>) -> Result<(), BadInstructionRange> {
        insn.check(&bad_instruction_ranges::UNSUPPORTED_BY_ARMAGNAC)?;
        insn.check(&bad_instruction_ranges::ILLEGAL)?;
        Ok(())
    }
    fn name() -> &'static str {
        "Armv6m"
    }
    fn proc() -> armagnac::core::Processor {
        armagnac::core::Processor::new(armagnac::core::Config::v6m())
    }
}

/// ArmV7E-M
#[derive(Clone)]
pub(crate) struct Armv7em;
impl Variant for Armv7em {
    fn check(insn: &ThumbInstruction<Self>) -> Result<(), BadInstructionRange> {
        insn.check(&bad_instruction_ranges::UNSUPPORTED_BY_ARMAGNAC)?;
        insn.check(&bad_instruction_ranges::ILLEGAL)?;
        Ok(())
    }
    fn name() -> &'static str {
        "Armv7em"
    }
    fn proc() -> armagnac::core::Processor {
        armagnac::core::Processor::new(armagnac::core::Config::v7em())
    }
}

/// ArmV7-M
#[derive(Clone)]
pub(crate) struct Armv7m;
impl Variant for Armv7m {
    fn check(insn: &ThumbInstruction<Self>) -> Result<(), BadInstructionRange> {
        insn.check(&bad_instruction_ranges::UNSUPPORTED_BY_ARMAGNAC)?;
        insn.check(&bad_instruction_ranges::ILLEGAL)?;
        Ok(())
    }
    fn name() -> &'static str {
        "Armv7m"
    }
    fn proc() -> armagnac::core::Processor {
        armagnac::core::Processor::new(armagnac::core::Config::v7m())
    }
}

/// ArmV8-M
#[derive(Clone)]
pub(crate) struct Armv8m;
impl Variant for Armv8m {
    fn check(insn: &ThumbInstruction<Self>) -> Result<(), BadInstructionRange> {
        insn.check(&bad_instruction_ranges::UNSUPPORTED_BY_ARMAGNAC)?;
        insn.check(&bad_instruction_ranges::ILLEGAL)?;
        // These are non-secure branches, not implemented by armagnac.
        insn.check(&[
            BadInstructionRange(0x4704..=0x4707),
            BadInstructionRange(0x470c..=0x470f),
            BadInstructionRange(0x4714..=0x4717),
            BadInstructionRange(0x471c..=0x471f),
            BadInstructionRange(0x4724..=0x4727),
            BadInstructionRange(0x472c..=0x472f),
            BadInstructionRange(0x4734..=0x4737),
            BadInstructionRange(0x473c..=0x473f),
            BadInstructionRange(0x4744..=0x4747),
            BadInstructionRange(0x474c..=0x474f),
            BadInstructionRange(0x4754..=0x4757),
            BadInstructionRange(0x475c..=0x475f),
            BadInstructionRange(0x4764..=0x4767),
            BadInstructionRange(0x476c..=0x476f),
            BadInstructionRange(0x4774..=0x4777),
            BadInstructionRange(0x477c..=0x477f),
            BadInstructionRange(0x4784..=0x4787),
            BadInstructionRange(0x478c..=0x478f),
            BadInstructionRange(0x4794..=0x4797),
            BadInstructionRange(0x479c..=0x479f),
            BadInstructionRange(0x47a4..=0x47a7),
            BadInstructionRange(0x47ac..=0x47af),
            BadInstructionRange(0x47b4..=0x47b7),
            BadInstructionRange(0x47bc..=0x47bf),
            BadInstructionRange(0x47c4..=0x47c7),
            BadInstructionRange(0x47cc..=0x47cf),
            BadInstructionRange(0x47d4..=0x47d7),
            BadInstructionRange(0x47dc..=0x47df),
            BadInstructionRange(0x47e4..=0x47e7),
            BadInstructionRange(0x47ec..=0x47ef),
            BadInstructionRange(0x47f4..=0x47f7),
            BadInstructionRange(0x47fc..=0x47ff),
        ])?;
        Ok(())
    }
    fn name() -> &'static str {
        "Armv8m"
    }
    fn proc() -> armagnac::core::Processor {
        armagnac::core::Processor::new(armagnac::core::Config::v8m())
    }
}
