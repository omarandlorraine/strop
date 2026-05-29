//! A backend targetting the ARMv4T architecture
//!
//! Coprocessors not supported; this will use softfloats

mod instruction_set;
pub use instruction_set::Instruction;
mod peephole;

pub mod aapcs32;

mod dataflow;

#[cfg(test)]
mod test;

/// Returns a ExplorationPoint, of ARM instructions, which makes sure to end in a return
/// instruction. Makes sure that relative branches are in range, and to avoid add/subtract
/// instructions with bounds checking.
pub fn subroutine() -> crate::search::platform_specific::ExplorationPoint<Instruction> {
    // TODO: understand relative branches and write something that makes sure the branches are in
    // range like what the MIPS one does.
    // TODO: rewrite the doc comment on this function
    crate::search::platform_specific::ExplorationPoint::<Instruction>::new()
        .add(|seq| seq.check_last(Instruction::make_bx_lr))
        .pointless(&[peephole::peephole_optimizer])
        .to_owned()
}
