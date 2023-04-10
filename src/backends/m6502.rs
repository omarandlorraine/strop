use crate::search::SearchConstraint;

use crate::randomly;
use crate::Emulator;
use crate::Instruction;
use asm::Decode;
use mos6502::cpu;
use rand::random;
use rand::seq::SliceRandom;

fn random_opcode() -> u8 {
    // All the opcodes.
    let opcodes = vec![
        0x00, 0x08, 0x10, 0x18, 0x20, 0x24, 0x28, 0x2c, 0x30, 0x38, 0x40, 0x48, 0x4c, 0x50, 0x58,
        0x60, 0x68, 0x6c, 0x70, 0x78, 0x84, 0x88, 0x8c, 0x90, 0x94, 0x98, 0xa0, 0xa4, 0xa8, 0xac,
        0xb0, 0xb4, 0xb8, 0xbc, 0xc0, 0xc4, 0xc8, 0xcc, 0xd0, 0xd8, 0xe0, 0xe4, 0xe8, 0xec, 0xf0,
        0xf8, 0x01, 0x05, 0x09, 0x0d, 0x11, 0x15, 0x19, 0x1d, 0x21, 0x25, 0x29, 0x2d, 0x31, 0x35,
        0x39, 0x3d, 0x41, 0x45, 0x49, 0x4d, 0x51, 0x55, 0x59, 0x5d, 0x61, 0x65, 0x69, 0x6d, 0x71,
        0x75, 0x79, 0x7d, 0x81, 0x85, 0x8d, 0x91, 0x95, 0x99, 0x9d, 0xa1, 0xa5, 0xa9, 0xad, 0xb1,
        0xb5, 0xb9, 0xbd, 0xc1, 0xc5, 0xc9, 0xcd, 0xd1, 0xd5, 0xd9, 0xdd, 0xe1, 0xe5, 0xe9, 0xed,
        0xf1, 0xf5, 0xf9, 0xfd, 0x06, 0x0a, 0x0e, 0x16, 0x1e, 0x26, 0x2a, 0x2e, 0x36, 0x3e, 0x46,
        0x4a, 0x4e, 0x56, 0x5e, 0x66, 0x6a, 0x6e, 0x76, 0x7e, 0x86, 0x8a, 0x8e, 0x96, 0x9a, 0xa2,
        0xa6, 0xaa, 0xae, 0xb6, 0xba, 0xbe, 0xc6, 0xca, 0xce, 0xd6, 0xde, 0xe6, 0xea, 0xee, 0xf6,
        0xfe,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_accumulator_opcode() -> u8 {
    // All the opcodes which have the Accumulator addressing mode.
    let opcodes = vec![0x0a, 0x4a, 0x2a, 0x6a];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_implied_opcode() -> u8 {
    // All the opcodes which have the Implied addressing mode.
    let opcodes = vec![
        0x18, 0x18, 0xd8, 0x58, 0xb8, 0xca, 0x88, 0xe8, 0xc8, 0xea, 0x48, 0x08, 0x68, 0x28, 0x60,
        0x38, 0xf8, 0x78, 0xaa, 0xa8, 0xba, 0x8a, 0x9a, 0x98, 0x40,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_immediate_opcode() -> u8 {
    // All the opcodes which have the Immediate addressing mode.
    let opcodes = vec![
        0x69, 0x29, 0xc9, 0xe0, 0xc0, 0x49, 0xa9, 0xa2, 0xa0, 0x09, 0xe9,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_zeropage_opcode() -> u8 {
    // All the opcodes which have the ZeroPage addressing mode.
    let opcodes = vec![
        0x65, 0x25, 0x06, 0x24, 0xC5, 0xE4, 0xC4, 0xC6, 0x45, 0xE6, 0xA5, 0xA6, 0xA4, 0x46, 0x05,
        0x26, 0x66, 0xE5, 0x85, 0x86, 0x84,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_zeropagex_opcode() -> u8 {
    // All the opcodes which have the ZeroPage,X addressing mode.
    let opcodes = vec![
        0x75, 0x35, 0x16, 0xd5, 0xd6, 0x55, 0xf6, 0xb5, 0xb4, 0x56, 0x15, 0x36, 0x76, 0xf5, 0x95,
        0x94,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_zeropagey_opcode() -> u8 {
    // All the opcodes which have the ZeroPage,Y addressing mode.
    let opcodes = vec![0xb6, 0x96];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_relative_opcode() -> u8 {
    // All the opcodes which have the Relative addressing mode.
    let opcodes = vec![0x90, 0xb0, 0xf0, 0x30, 0xd0, 0x10, 0x50, 0x70];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_absolute_opcode() -> u8 {
    // All the opcodes which have the Absolute addressing mode.
    let opcodes = vec![
        0x6d, 0x2d, 0x0e, 0x2c, 0xcd, 0xec, 0xcc, 0xce, 0x4d, 0xee, 0x4c, 0x20, 0xad, 0xae, 0xac,
        0x4e, 0x0d, 0x2e, 0x6e, 0xed, 0x8d, 0x8e, 0x8c,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_absolutex_opcode() -> u8 {
    // All the opcodes which have the Absolute,X addressing mode.
    let opcodes = vec![
        0x7d, 0x3d, 0x1e, 0xdd, 0xde, 0x5d, 0xfe, 0xbd, 0xbc, 0x5e, 0x1d, 0x3e, 0x7e, 0xfd, 0x9d,
    ];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_absolutey_opcode() -> u8 {
    // All the opcodes which have the Absolute,Y addressing mode.
    let opcodes = vec![0x79, 0x39, 0xd9, 0x59, 0xb9, 0xbe, 0x19, 0xf9, 0x99];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_indirect_opcode() -> u8 {
    // All the opcodes which have the Indirect addressing mode.
    0x6c
}

fn random_indirectx_opcode() -> u8 {
    // All the opcodes which have the Indirect,X addressing mode.
    let opcodes = vec![0x61, 0x21, 0xc1, 0x41, 0xa1, 0x01, 0xe1, 0x81];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

fn random_indirecty_opcode() -> u8 {
    // All the opcodes which have the Indirect,Y addressing mode.
    let opcodes = vec![0x71, 0x31, 0xd1, 0x51, 0xb1, 0x11, 0xf1, 0x91];
    *opcodes.choose(&mut rand::thread_rng()).unwrap()
}

#[derive(Clone, Copy)]
pub struct Mos6502Instruction {
    internal: asm::_6502::Instruction,
}

impl Mos6502Instruction {
    fn set_opcode(&mut self, op: u8) {
        use asm::Encode;
        use asm::_6502::{Decoder, Encoder};
        let mut dasm = [0u8; 3];
        let mut encoder = Encoder::new(&mut dasm[..]);
        encoder.encode(self.internal).unwrap();
        dasm[0] = op;
        let mut decoder = Decoder::new(&dasm[..]);
        self.internal = decoder.decode().unwrap();
    }

    fn mutate_opcode(&mut self) {
        match self.internal.addressing() {
            asm::_6502::Addressing::Accumulator => self.set_opcode(random_accumulator_opcode()),
            asm::_6502::Addressing::Implied => self.set_opcode(random_implied_opcode()),
            asm::_6502::Addressing::Immediate(_) => self.set_opcode(random_immediate_opcode()),
            asm::_6502::Addressing::ZeroPage(_) => self.set_opcode(random_zeropage_opcode()),
            asm::_6502::Addressing::ZeroPageX(_) => self.set_opcode(random_zeropagex_opcode()),
            asm::_6502::Addressing::ZeroPageY(_) => self.set_opcode(random_zeropagey_opcode()),
            asm::_6502::Addressing::Relative(_) => self.set_opcode(random_relative_opcode()),
            asm::_6502::Addressing::Absolute(_) => self.set_opcode(random_absolute_opcode()),
            asm::_6502::Addressing::AbsoluteX(_) => self.set_opcode(random_absolutex_opcode()),
            asm::_6502::Addressing::AbsoluteY(_) => self.set_opcode(random_absolutey_opcode()),
            asm::_6502::Addressing::Indirect(_) => self.set_opcode(random_indirect_opcode()),
            asm::_6502::Addressing::IndexedIndirect(_) => {
                self.set_opcode(random_indirectx_opcode())
            }
            asm::_6502::Addressing::IndirectIndexed(_) => {
                self.set_opcode(random_indirecty_opcode())
            }
        }
    }

    fn mutate_operand(&mut self) {
        // This method picks one of the two bytes of the instruction's opcode, and then either
        // increments it, decrements it, or flips a random bit.
        use asm::Encode;
        use asm::_6502::{Decoder, Encoder};
        let mut dasm = [0u8; 3];
        let mut encoder = Encoder::new(&mut dasm[..]);
        encoder.encode(self.internal).unwrap();

        let offs = if random() { 1 } else { 2 };
        let bitsel = randomly!({0x01} {0x02} {0x04} {0x08} {0x10} {0x20} {0x40} {0x80});
        randomly!(
            {dasm[offs] += 1}
            {dasm[offs] -= 1}
            {dasm[offs] ^= bitsel}
        );

        let mut decoder = Decoder::new(&dasm[..]);
        self.internal = decoder.decode().unwrap();
    }
}

impl Instruction for Mos6502Instruction {
    fn new() -> Self {
        let istream = [random_opcode(), random(), random()];
        let mut decoder = asm::_6502::Decoder::new(&istream[..]);
        let internal = decoder.decode().unwrap();
        Self { internal }
    }

    fn mutate(&mut self) {
        randomly!(
            { self.mutate_operand() }
            { self.mutate_opcode() }
        )
    }

    fn length(&self) -> usize {
        self.internal.length()
    }

    fn disassemble(&self) -> String {
        format!("{:?}", self.internal)
    }
}

pub struct Mos6502Emulator {
    internal: cpu::CPU,
}

impl Default for Mos6502Emulator {
    fn default() -> Self {
        Self {
            internal: cpu::CPU::new(),
        }
    }
}

impl Emulator for Mos6502Emulator {
    type Addr = u16;
    type Insn = u8;

    fn run(&mut self, org: Self::Addr, prog: &[Self::Insn]) {
        use mos6502::address::Address;
        self.internal.memory.set_bytes(Address(org), prog);
        todo!("We need to actually run the program");
    }
}

pub struct SearchConstraint6502 {
    accept: fn(Mos6502Instruction) -> bool,
    parent: Box<Self>
}

fn no_decimal_mode(insn: Mos6502Instruction) -> bool {
    /// returns false for instructions that manipulate the decimal flag (and which will not appear
    /// in programs behaving sensibly on the Ricoh 2A03)
    use  asm::_6502::Instruction;
    match insn.internal {
        Instruction::CLD(_) => false,
        Instruction::SED(_) => false,
        _ => true,
    }
}

fn no_ror(insn: Mos6502Instruction) -> bool {
    /// returns false for the ROR instruction, which on very early specimens are not present due to
    /// a hardware bug
    use  asm::_6502::Instruction;
    match insn.internal {
        Instruction::ROR(_) => false,
        _ => true,
    }
}

fn basic_block(insn: Mos6502Instruction) -> bool {
    /// returns true only for instructions that are allowed inside a basic block
    use  asm::_6502::Instruction;
    match insn.internal {
        Instruction::BCC(_) => false,
        Instruction::BCS(_) => false,
        Instruction::BEQ(_) => false,
        Instruction::BMI(_) => false,
        Instruction::BNE(_) => false,
        Instruction::BRK(_) => false,
        Instruction::BPL(_) => false,
        Instruction::BRK(_) => false,
        Instruction::BVC(_) => false,
        Instruction::BVS(_) => false,
        Instruction::JMP(_) => false,
        Instruction::JSR(_) => false,
        Instruction::RTI(_) => false,
        Instruction::RTS(_) => false,
        _ => true,
    }
}

impl SearchConstraint6502 {
    fn basic_block(self) -> Self {
        Self {parent: Box::new(self), accept: basic_block }
    }

    fn no_decimal_mode(self) -> Self {
        Self {parent: Box::new(self), accept: no_decimal_mode }
    }

    fn no_ror(self) -> Self {
        Self {parent: Box::new(self), accept: no_ror }
    }
}

impl SearchConstraint<Mos6502Instruction> for SearchConstraint6502 {
    fn mutate(&self, t: Mos6502Instruction) -> Mos6502Instruction {
        if (self.accept)(t) {
            self.parent.mutate(t)
        } else {
            self.mutate(Mos6502Instruction::new())
        }
    }
}
