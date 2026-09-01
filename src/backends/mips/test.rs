use crate::Instruction as _;
use crate::backends::mips;
use mips::Instruction;

fn single_step_over_instruction(insn: &Instruction) {
    use trapezoid_core::cpu::RegisterType;

    let mut cpu = trapezoid_core::cpu::Cpu::new();
    let mut bus = crate::backends::mips::bus::Bus::new();

    // writes the instruction to the beginning of kseg1, and then single steps across it
    println!("single step: {:?}", insn);
    bus.kseg1[0..4].copy_from_slice(&insn.to_bytes());
    cpu.registers_mut().write(RegisterType::Pc, 0xBFC00000);
    cpu.clock(&mut bus, 1);
}

/// Increments the instruction to the next instruction worth testing. This skips all but one
/// possible values for the 16-bit immediate value, etc. Thus a test can complete in a reasonable
/// amount of time.
fn next_instruction_worth_testing(insn: &mut Instruction) -> crate::IterationResult {
    if insn.r() {
        insn.increment()
    } else if insn.i() {
        insn.next_registers()
    } else {
        insn.next_opcode()
    }
}

#[ignore]
#[test]
fn disassemblies_unique() {
    crate::generic_unit_tests::disassemblies_unique(
        Instruction::from_bytes(&[0, 0, 0, 0]).unwrap(),
        None,
    );
}

#[test]
fn trapezoid_support() {
    let mut insn = Instruction::first();

    loop {
        single_step_over_instruction(&insn);
        if next_instruction_worth_testing(&mut insn).is_err() {
            break;
        }
    }
}
