/// Represents a 6502 machine instruction, compatible with some 6502 variant.
#[derive(Copy, Clone, Default, PartialOrd, PartialEq)]
pub struct Insn<V: mos6502::Variant + std::clone::Clone>([u8; 3], std::marker::PhantomData<V>)
where
    V: std::clone::Clone;

impl<V: mos6502::Variant + std::clone::Clone> crate::Iterable for Insn<V> {
    fn first() -> Self {
        Self([0, 0, 0], std::marker::PhantomData::<V>)
    }

    fn step(&mut self) -> bool {
        use crate::Encode;
        self.incr_at_offset(self.len() - 1);
        self.fixup();

        // check if we reached the last opcode
        // ($ff is not a valid opcode)
        self.0[0] != 0xff
    }
}

impl<V: mos6502::Variant + std::clone::Clone> Insn<V> {
    /// Increments the opcode, and sets all subsequent bytes (i.e. the operand) to 0.
    pub fn skip_to_next_opcode(&mut self) -> bool {
        self.0 = [self.0[0] + 1, 0, 0];
        self.fixup();

        // check if we reached the last opcode
        // ($ff is not a valid opcode)
        self.0[0] != 0xff
    }
}

impl<V: mos6502::Variant + std::clone::Clone> crate::Encode<u8> for Insn<V> {
    fn encode(&self) -> std::vec::Vec<u8> {
        let mut encoding = self.0.to_vec();
        encoding.truncate(self.len());
        encoding
    }

    fn len(&self) -> usize {
        use mos6502::instruction::AddressingMode;
        match self.decode().1 {
            AddressingMode::Accumulator => 1,
            AddressingMode::Implied => 1,
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 2,
            AddressingMode::ZeroPageY => 2,
            AddressingMode::ZeroPageIndirect => 2,
            AddressingMode::Relative => 2,
            AddressingMode::IndexedIndirectX => 2,
            AddressingMode::IndirectIndexedY => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::AbsoluteX => 3,
            AddressingMode::AbsoluteY => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::BuggyIndirect => 3,
        }
    }
}

impl<V: mos6502::Variant + std::clone::Clone> Insn<V> {
    /// constructs a return instruction `rts`.
    pub fn rts() -> Self {
        Self::new(&[0x60])
    }

    /// Constructs a new Insn from a slice of bytes
    pub fn new(mc: &[u8]) -> Self {
        let mut enc = [0, 0, 0];
        enc[..mc.len().min(3)].copy_from_slice(mc);
        Self(enc, std::marker::PhantomData::<V>)
    }

    fn incr_at_offset(&mut self, offset: usize) -> bool {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
            true
        } else {
            self.0[offset] = 0;
            if offset == 0 {
                false
            } else {
                self.incr_at_offset(offset - 1)
            }
        }
    }

    /// Decodes and parses the instruction and returns miscellaneous information about it.
    pub fn decode(
        &self,
    ) -> (
        mos6502::instruction::Instruction,
        mos6502::instruction::AddressingMode,
    ) {
        use mos6502::Variant;
        mos6502::instruction::Cmos6502::decode(self.0[0])
            .unwrap_or_else(|| panic!("can't decode opcode {:02x}", self.0[0]))
    }

    /// If this instruction's operand is an address, then this method returns that address.
    pub fn address(&self) -> Option<u16> {
        use mos6502::instruction::AddressingMode;

        match self.decode().1 {
            AddressingMode::Accumulator
            | AddressingMode::Implied
            | AddressingMode::Immediate
            | AddressingMode::Relative => None,

            AddressingMode::ZeroPage
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::ZeroPageIndirect
            | AddressingMode::IndexedIndirectX
            | AddressingMode::IndirectIndexedY => Some(u16::from_le_bytes([self.0[1], 0])),

            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect
            | AddressingMode::BuggyIndirect => Some(u16::from_le_bytes([self.0[1], self.0[2]])),
        }
    }

    /// If this instruction writes to an address, this method returns that address.
    pub fn writes_to(&self) -> Option<u16> {
        use mos6502::instruction::Instruction::*;

        match self.decode().0 {
            // ALU operations don't write to memory
            ADC | ADCnd | AND | BIT | EOR | CMP | CPX | CPY | ORA | SBC | SBCnd => None,

            // flow control instructions don't write to memory
            BVC | BVS | BNE | BPL | BRA | BCC | BCS | BEQ | BMI => None,
            BRK | BRKcld | JMP | JSR | RTS | RTI => None,

            // setting and clearing flags won't write to memory
            CLC | CLD | CLI | CLV | SEC | SED | SEI => None,

            // Read-modify-write instructions do write to memory.
            INC | DEC | ROR | ROL | ASL | LSR | TSB | TRB => self.address(),

            // (but not if they operate on X or Y instead)
            DEX | DEY | INX | INY => None,

            // load instructions don't write to memory
            LDA | LDY | LDX => None,

            // nop obviously doesn't either
            NOP => None,

            // pushing and pulling to stack doesn't count
            PHA | PHX | PHY | PHP | PLA | PLX | PLY | PLP => None,

            // store instructions write to memory
            STA | STX | STY | STZ => self.address(),

            // register-to-register operations don't write to memory
            TAX | TAY | TXA | TYA | TSX | TXS => None,
        }
    }

    /// If this instruction reads from an address, this method returns that address.
    pub fn reads_from(&self) -> Option<u16> {
        use mos6502::instruction::Instruction::*;

        match self.decode().0 {
            // ALU operations read from memory
            ADC | ADCnd | AND | BIT | EOR | CMP | CPX | CPY | ORA | SBC | SBCnd => self.address(),

            // flow control instructions don't read from memory
            BVC | BVS | BNE | BPL | BRA | BCC | BCS | BEQ | BMI => None,
            BRK | BRKcld | JMP | JSR | RTS | RTI => None,

            // setting and clearing flags won't read from memory
            CLC | CLD | CLI | CLV | SEC | SED | SEI => None,

            // Read-modify-write instructions do read from memory
            INC | DEC | ROR | ROL | ASL | LSR | TSB | TRB => self.address(),

            // (but not if they operate on X or Y instead)
            DEX | DEY | INX | INY => None,

            // load instructions read from memory
            LDA | LDY | LDX => self.address(),

            // nop obviously doesn't either
            NOP => None,

            // pushing and pulling to stack doesn't count
            PHA | PHX | PHY | PHP | PLA | PLX | PLY | PLP => None,

            // store instructions don't read from memory
            STA | STX | STY | STZ => None,

            // register-to-register operations don't read from memory
            TAX | TAY | TXA | TYA | TSX | TXS => None,
        }
    }
    /// Regardless of the `Insn`'s current value, this mutates it such that it now represents a
    /// valid 6502 machine instruction. Be sure that the opcode field is not beyond the range of
    /// valid opcodes.
    pub fn fixup(&mut self) {
        use mos6502::Variant;
        while mos6502::instruction::Cmos6502::decode(self.0[0]).is_none() {
            if let Some(new_opcode) = self.0[0].checked_add(1) {
                self.0[0] = new_opcode;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn first_few() {
        use super::Insn;
        use crate::Iterable;
        use mos6502::instruction::Cmos6502;

        let mut i = Insn::<Cmos6502>::first();
        println!("{} ; {:?}", i, i);
        assert!(i.step());
        println!("{} ; {:?}", i, i);
        assert!(i.step());
        println!("{} ; {:?}", i, i);
        assert!(i.step());
        println!("{} ; {:?}", i, i);
        assert!(i.step());
    }

    #[test]
    fn all_instructions() {
        use super::Insn;
        use crate::Iterable;
        use mos6502::instruction::Cmos6502;

        let mut i = Insn::<Cmos6502>::first();

        while i.step() {
            println!("{} ; {:?}", i, i);
            i.decode();
        }
    }
}
