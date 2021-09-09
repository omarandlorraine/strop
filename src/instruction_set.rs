mod instruction_set;
use crate::machine::Instruction;
use crate::machine::AddressingMode;

const basic6502: &'static [&'static Instruction] = &[
{mode:AddressingMode::Implicit, operation:Operation::INX},
{mode:AddressingMode::Implicit, operation:Operation::INY},
{mode:AddressingMode::Implicit, operation:Operation::DEX},
{mode:AddressingMode::Implicit, operation:Operation::DEY},
{mode:AddressingMode::Implicit, operation:Operation::TAX},
{mode:AddressingMode::Implicit, operation:Operation::TAY},
{mode:AddressingMode::Implicit, operation:Operation::TXA},
{mode:AddressingMode::Implicit, operation:Operation::TYA},
]

const basic6800: &'static [&'static Instruction] = &[
{mode:AddressingMode::Implicit, operation:Operation::ABA},
{mode:AddressingMode::Implicit, operation:Operation::TAB},
{mode:AddressingMode::Implicit, operation:Operation::TBA},
]


