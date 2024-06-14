//! Module containing a peephole optimizer for 6502 programs. The purpose of this peephole
//! optimizer is to reduce the search space explored by the Bruteforce search alogorithm.
use crate::mos6502::Cmos6502Instruction;
use crate::Fixup;
use crate::Instruction;
use crate::Peephole;

// TODO: there's some duplicate code here,

#[derive(Debug)]
struct NextOpcode;

impl Fixup<Cmos6502Instruction> for NextOpcode {
    fn random(&self, _insn: Cmos6502Instruction) -> Cmos6502Instruction {
        panic!();
    }

    fn next(&self, insn: Cmos6502Instruction) -> Option<Cmos6502Instruction> {
        insn.increment_opcode()
    }

    fn check(&self, _insn: Cmos6502Instruction) -> bool {
        true
    }
}

fn get_first_two_instructions(
    insns: &[Cmos6502Instruction],
) -> Option<(Cmos6502Instruction, Cmos6502Instruction)> {
    if insns.len() > 1 {
        Some((insns[0], insns[1]))
    } else {
        None
    }
}

fn pointless_instruction_sequence(
    insns: &[Cmos6502Instruction],
) -> Option<(Box<dyn Fixup<Cmos6502Instruction>>, usize)> {
    // Sequences of instructions that are not meaningful are caught by this function. These are
    // sequences such as,
    // - anything beginning with a `nop`,
    // - anything that sets and then clears a flag, or vice versa
    // - anything that increments and then decrements a register (this does affect flags; but a
    //   compare instruction is better)

    let (first, second) = get_first_two_instructions(insns)?;
    let (first, second) = (first.encode()[0], second.encode()[1]);

    let groups: Vec<Vec<u8>> = vec![
        vec![0x18, 0x38], // clc, sec
        vec![0x58, 0x78], // cli, sei
        vec![0xd8, 0xf8], // cld, sed
    ];

    for group in groups {
        if group.contains(&first) && group.contains(&second) {
            // we've got two instructions in the same group, so the first instruction is pointless
            return Some((Box::new(NextOpcode), 0));
        }
    }
    None
}

fn independent_instructions_are_in_order(
    insns: &[Cmos6502Instruction],
) -> Option<(Box<dyn Fixup<Cmos6502Instruction>>, usize)> {
    // Where instructions in a group may be equivalently executed in an arbitrary order, I say that
    // we may as well have these instructions in a canonical order. That canonical order is numerical.
    // This means that the Bruteforce search would not consider, for example, both of these
    // programs:
    // - sec; tax
    // - tax; sec
    // because they are equivalent, because the two instructions do not depend on eachother.

    let (first, second) = get_first_two_instructions(insns)?;
    let (first, second) = (first.encode()[0], second.encode()[1]);

    if first < second {
        // the instructions are in order, so there's no need to check anything else.
        return None;
    }

    let groups: Vec<Vec<u8>> = vec![
        // instructions that set or clear flags are independent of eachother (unless the sequence
        // has been caught by "pointless_instruction_sequence"!)
        vec![0x18, 0x38, 0x58, 0x78, 0xb8, 0xd8, 0xf8],
        // instructions moving data out of the accumulator are independent of eachother
        // (this includes `pha`, `tax`, `tay`, `sta`
        vec![0xa8, 0xaa, 0x48, 0x85, 0x95, 0x8d, 0x9d, 0x99, 0x81, 0x91],
        // instructions moving data out of X are independent of eachother
        // (this includes `phx`, `txa`, `txs`, `stx`
        vec![0x8a, 0x9a, 0xda, 0x86, 0x96, 0x8e],
        // instructions moving data out of y are independent of eachother
        // (this includes `phy`, `tya`, `sty`
        vec![0x5a, 0x98, 0x84, 0x94, 0x8c],
    ];

    for group in groups {
        if group.contains(&first) && group.contains(&second) {
            // we've got two instructions in the same group but not in the canonical order
            return Some((Box::new(NextOpcode), 0));
        }
    }
    None
}

impl Peephole for Cmos6502Instruction {
    fn peephole(
        insns: &[Cmos6502Instruction],
    ) -> Option<(Box<dyn Fixup<Cmos6502Instruction>>, usize)> {
        if let Some(ans) = pointless_instruction_sequence(insns) {
            return Some(ans);
        }
        if let Some(ans) = independent_instructions_are_in_order(insns) {
            return Some(ans);
        }
        None
    }
}
