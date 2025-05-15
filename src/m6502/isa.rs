use crate::IterationResult;
use crate::StepError;
/// Represents a 6502 machine instruction, compatible with some 6502 variant.
#[derive(Copy, Clone, Default, PartialOrd, PartialEq)]
pub struct Insn<V: mos6502::Variant>([u8; 3], std::marker::PhantomData<V>);

impl<V: mos6502::Variant> crate::Step for Insn<V> {
    fn first() -> Self {
        Self([0, 0, 0], std::marker::PhantomData::<V>)
    }

    fn next(&mut self) -> IterationResult {
        use crate::Encode;
        self.incr_at_offset(self.len() - 1);
        self.fixup()
    }
}

impl<V: mos6502::Variant> crate::subroutine::ShouldReturn for Insn<V> {
    fn should_return(&self) -> Result<(), crate::StaticAnalysis<Self>> {
        if self.0 == Self::rts().0 {
            return Ok(());
        }
        Err(crate::StaticAnalysis::<Self> {
            offset: 0,
            advance: Self::skip_to_next_opcode,
            reason: "ShouldReturn",
        })
    }
}

impl<V: mos6502::Variant> Insn<V> {
    /// Increments the opcode, and sets all subsequent bytes (i.e. the operand) to 0.
    pub fn skip_to_next_opcode(&mut self) -> IterationResult {
        self.0 = [self.0[0] + 1, 0, 0];
        self.fixup()
    }
}

impl<V: mos6502::Variant> crate::Branch for Insn<V> {}

impl<V: mos6502::Variant> crate::Encode<u8> for Insn<V> {
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

impl<V: mos6502::Variant> Insn<V> {
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
    /// valid 6502 machine instruction.
    pub fn fixup(&mut self) -> IterationResult {
        while V::decode(self.0[0]).is_none() {
            if let Some(new_opcode) = self.0[0].checked_add(1) {
                self.0[0] = new_opcode;
            } else {
                return Err(StepError::End);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    fn check_insn<V>(insn: super::Insn<V>)
    where
        V: Clone + mos6502::Variant,
    {
        use crate::Encode;
        insn.decode();
        let mut copy = insn.clone();
        copy.fixup().unwrap();
        assert_eq!(copy.encode(), insn.encode());
    }

    #[test]
    fn first_few() {
        use super::Insn;
        use crate::Step;
        use mos6502::instruction::Cmos6502;

        let mut i = Insn::<Cmos6502>::first();
        println!("{} ; {:?}", i, i);
        assert!(i.next().is_ok());
        println!("{} ; {:?}", i, i);
        assert!(i.next().is_ok());
        println!("{} ; {:?}", i, i);
        assert!(i.next().is_ok());
        println!("{} ; {:?}", i, i);
        assert!(i.next().is_ok());
    }

    fn all_instructions<V>()
    where
        V: Clone + mos6502::Variant,
    {
        use super::Insn;
        use crate::Step;

        let mut i = Insn::<V>::first();

        let mut count = 0usize;

        while i.next().is_ok() {
            check_insn(i.clone());
            count += 1;
        }
        println!("Iterated over {count} instructions");
    }

    #[test]
    fn all_instructions_cmos() {
        all_instructions::<mos6502::instruction::Cmos6502>();
    }

    #[test]
    fn all_instructions_reva() {
        all_instructions::<mos6502::instruction::RevisionA>();
    }

    #[test]
    fn all_instructions_2a03() {
        all_instructions::<mos6502::instruction::Ricoh2a03>();
    }

    #[test]
    fn all_instructions_nmos() {
        all_instructions::<mos6502::instruction::Nmos6502>();
    }
}
