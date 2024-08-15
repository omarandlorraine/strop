impl<V: mos6502::Variant + std::clone::Clone> std::fmt::Display for crate::m6502::isa::Insn<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Encode;
        use mos6502::instruction::AddressingMode;
        use mos6502::instruction::Instruction;

        let d = self.decode();

        match d.0 {
            Instruction::ADC => write!(f, "adc"),
            Instruction::ADCnd => write!(f, "adc"),
            Instruction::AND => write!(f, "and"),
            Instruction::ASL => write!(f, "asl"),
            Instruction::BCC => write!(f, "bcc"),
            Instruction::BCS => write!(f, "bcs"),
            Instruction::BEQ => write!(f, "beq"),
            Instruction::BIT => write!(f, "bit"),
            Instruction::BMI => write!(f, "bmi"),
            Instruction::BNE => write!(f, "bne"),
            Instruction::BPL => write!(f, "bpl"),
            Instruction::BRA => write!(f, "bra"),
            Instruction::BRK => write!(f, "brk"),
            Instruction::BRKcld => write!(f, "brk"),
            Instruction::BVC => write!(f, "bvc"),
            Instruction::BVS => write!(f, "bvs"),
            Instruction::CLC => write!(f, "clc"),
            Instruction::CLD => write!(f, "cld"),
            Instruction::CLI => write!(f, "cli"),
            Instruction::CLV => write!(f, "clv"),
            Instruction::CMP => write!(f, "cmp"),
            Instruction::CPX => write!(f, "cpx"),
            Instruction::CPY => write!(f, "cpy"),
            Instruction::DEC => write!(f, "dec"),
            Instruction::DEX => write!(f, "dex"),
            Instruction::DEY => write!(f, "dey"),
            Instruction::EOR => write!(f, "eor"),
            Instruction::INC => write!(f, "inc"),
            Instruction::INX => write!(f, "inx"),
            Instruction::INY => write!(f, "iny"),
            Instruction::JMP => write!(f, "jmp"),
            Instruction::JSR => write!(f, "jsr"),
            Instruction::LDA => write!(f, "lda"),
            Instruction::LDX => write!(f, "ldx"),
            Instruction::LDY => write!(f, "ldy"),
            Instruction::LSR => write!(f, "lsr"),
            Instruction::NOP => write!(f, "nop"),
            Instruction::ORA => write!(f, "ora"),
            Instruction::PHA => write!(f, "pha"),
            Instruction::PHP => write!(f, "php"),
            Instruction::PHX => write!(f, "phx"),
            Instruction::PHY => write!(f, "phy"),
            Instruction::PLA => write!(f, "pla"),
            Instruction::PLP => write!(f, "plp"),
            Instruction::PLX => write!(f, "plx"),
            Instruction::PLY => write!(f, "ply"),
            Instruction::ROL => write!(f, "rol"),
            Instruction::ROR => write!(f, "ror"),
            Instruction::RTI => write!(f, "rti"),
            Instruction::RTS => write!(f, "rts"),
            Instruction::SBC => write!(f, "sbc"),
            Instruction::SBCnd => write!(f, "sbc"),
            Instruction::SEC => write!(f, "sec"),
            Instruction::SED => write!(f, "sed"),
            Instruction::SEI => write!(f, "sei"),
            Instruction::STA => write!(f, "sta"),
            Instruction::STX => write!(f, "stx"),
            Instruction::STY => write!(f, "sty"),
            Instruction::STZ => write!(f, "stz"),
            Instruction::TAX => write!(f, "tax"),
            Instruction::TAY => write!(f, "tay"),
            Instruction::TSX => write!(f, "tsx"),
            Instruction::TSB => write!(f, "tsb"),
            Instruction::TRB => write!(f, "trb"),
            Instruction::TXA => write!(f, "txa"),
            Instruction::TXS => write!(f, "txs"),
            Instruction::TYA => write!(f, "tya"),
        }?;

        match d.1 {
            AddressingMode::Accumulator => write!(f, " a"),
            AddressingMode::Implied => write!(f, ""),
            AddressingMode::Immediate => write!(f, " #${:02x}", self.encode()[1]),
            AddressingMode::ZeroPage => write!(f, " ${:02x}", self.encode()[1]),
            AddressingMode::ZeroPageX => write!(f, " ${:02x}, x", self.encode()[1]),
            AddressingMode::ZeroPageY => write!(f, " ${:02x}, y", self.encode()[1]),
            AddressingMode::Relative => write!(f, " {}", self.encode()[1] as i8),
            AddressingMode::Absolute => {
                write!(f, " ${:02x}{:02x}", self.encode()[1], self.encode()[2])
            }
            AddressingMode::AbsoluteX => {
                write!(f, " ${:02x}{:02x}, x", self.encode()[1], self.encode()[2])
            }
            AddressingMode::AbsoluteY => {
                write!(f, " ${:02x}{:02x}, y", self.encode()[1], self.encode()[2])
            }
            AddressingMode::Indirect | mos6502::instruction::AddressingMode::BuggyIndirect => {
                write!(f, " (${:02x}{:02x})", self.encode()[1], self.encode()[2])
            }
            AddressingMode::ZeroPageIndirect => {
                write!(f, " (${:02x})", self.encode()[1])
            }
            AddressingMode::IndexedIndirectX => write!(f, " (${:02x},x)", self.encode()[1]),
            AddressingMode::IndirectIndexedY => write!(f, " (${:02x}),y", self.encode()[1]),
        }?;
        Ok(())
    }
}

impl<V: mos6502::Variant + std::clone::Clone> std::fmt::Debug for crate::m6502::isa::Insn<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Encode;

        let bytes = self
            .encode()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{}", bytes)
    }
}
