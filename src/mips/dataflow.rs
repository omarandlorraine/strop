use crate::Sequence;
use crate::mips::Insn;
use trapezoid_core::cpu::{BusLine, Cpu, CpuBusProvider, RegisterType};

pub fn number_of_arguments(seq: &Sequence<Insn>, qty: usize) -> crate::StaticAnalysis<Insn> {
    let regs = [
        RegisterType::A0,
        RegisterType::A1,
        RegisterType::A2,
        RegisterType::A3,
    ];

    for reg in &regs[..=qty] {
        crate::dataflow::expect_read(seq, &reg)?;
    }
    for reg in &regs[qty..] {
        crate::dataflow::uninitialized(seq, &reg)?;
    }
    Ok(())
}

pub fn number_of_return_values(seq: &Sequence<Insn>, qty: usize) -> crate::StaticAnalysis<Insn> {
    let regs = [RegisterType::V0, RegisterType::V1];

    for reg in &regs[..=qty] {
        crate::dataflow::expect_read(seq, &reg)?;
    }
    for reg in &regs[qty..] {
        crate::dataflow::uninitialized(seq, &reg)?;
    }
    Ok(())
}

pub fn put1(cpu: &mut trapezoid_core::cpu::Cpu, val: u32) {
    cpu.registers_mut().write(RegisterType::A0, val);
}

pub fn put2(cpu: &mut trapezoid_core::cpu::Cpu, a: u32, b: u32) {
    cpu.registers_mut().write(RegisterType::A0, a);
    cpu.registers_mut().write(RegisterType::A0, b);
}

pub fn put3(cpu: &mut trapezoid_core::cpu::Cpu, a: u32, b: u32, c: u32) {
    cpu.registers_mut().write(RegisterType::A0, a);
    cpu.registers_mut().write(RegisterType::A0, b);
    cpu.registers_mut().write(RegisterType::A0, c);
}

pub fn get1(cpu: &trapezoid_core::cpu::Cpu) -> u32 {
cpu.registers().read(RegisterType::V0)
}

pub fn get2(cpu: &trapezoid_core::cpu::Cpu) -> (u32, u32) {
    (cpu.registers().read(RegisterType::V0), cpu.registers().read(RegisterType::V1))
}
