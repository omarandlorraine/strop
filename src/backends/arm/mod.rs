//! A backend for strop, targetting 32-bit ARMs.

mod arm_instructions;
mod cpu;
mod thumb_instructions;
use crate::StaticAnalysis;
pub use arm_instructions::ArmInstruction;
pub use thumb_instructions::ThumbInstruction;

/// The CPU variant. Checks which instructions are available
#[derive(Debug, Clone)]
pub struct Variant {
    version: unarm::Version,
    has_vfpdv2: bool,
}

impl Variant {
    fn options(&self) -> unarm::Options {
        use unarm::{Extension, Extensions};
        unarm::Options {
            av: false,
            extensions: if self.has_vfpdv2 {
                Extensions::of(&[Extension::VfpV2])
            } else {
                Extensions::none()
            },
            fp: false,
            ip: false,
            r9_use: Default::default(),
            sl: false,
            ual: false,
            version: self.version,
        }
    }
    
    /// Checks a Thumb instruction, to see if it is present on the given CPU variant
    pub fn check_thumb(&self, ins: &ThumbInstruction) -> StaticAnalysis<ArmInstruction> {
        use crate::Fixup;
        use unarm::Ins;
        let pc = 0;
        let (ins, _size) = unarm::parse_thumb(ins.codepoint(), pc, &self.options());
        match ins {
            Ins::Illegal => Err(Fixup::<ArmInstruction>::new(
                "InstructionNotSupported",
                ArmInstruction::skip_this_instruction,
                0,
            )),
            _ => Ok(()),
        }
    }
    
    /// Checks an ARM instruction, to see if it is present on the given CPU variant
    pub fn check_arm(&self, ins: &ArmInstruction) -> StaticAnalysis<ArmInstruction> {
        use crate::Fixup;
        use unarm::Ins;
        let pc = 0;
        match unarm::parse_arm(ins.codepoint(), pc, &self.options()) {
            Ins::Illegal => Err(Fixup::<ArmInstruction>::new(
                "InstructionNotSupported",
                ArmInstruction::skip_this_instruction,
                0,
            )),
            _ => Ok(()),
        }
    }
}
