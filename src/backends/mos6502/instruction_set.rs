//! Represents a 6502 machine instruction, compatible with some 6502 variant.

use crate::IterationResult;
use mos6502::instruction::AddressingMode;
use mos6502::instruction::Instruction as Opcode;

/// Represents a machine instruction of some variant of the 6502.
#[derive(Copy, Clone, Default, PartialOrd, PartialEq)]
pub struct Instruction<V: mos6502::Variant>([u8; 3], std::marker::PhantomData<V>);

impl<V: mos6502::Variant> Instruction<V> {
    fn valid_opcode(&self) -> bool {
        V::decode(self.0[0]).is_some()
    }
    fn len(&self) -> usize {
        match self.addressing_mode() {
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
            AddressingMode::AbsoluteIndexedIndirect => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::BuggyIndirect => 3,
        }
    }

    fn incr_at_offset(&mut self, offset: usize) -> IterationResult {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
            if offset == 0 {
                // skip invalid opcodes
                if V::decode(self.0[0]).is_none() {
                    self.incr_at_offset(0)?;
                }
            }
            Ok(())
        } else {
            self.0[offset] = 0;
            if offset == 0 {
                Err(crate::StepError::End)
            } else {
                self.incr_at_offset(offset - 1)
            }
        }
    }

    fn addressing_mode(&self) -> mos6502::instruction::AddressingMode {
        V::decode(self.0[0]).unwrap().1
    }

    fn opcode(&self) -> Opcode {
        V::decode(self.0[0]).unwrap().0
    }

    fn skip_opcode(&mut self) -> IterationResult {
        self.incr_at_offset(0)
    }

    fn skip_operand(&mut self) -> IterationResult {
        match self.len() {
            1 => self.incr_at_offset(0),
            2 => self.incr_at_offset(1),
            3 => self.incr_at_offset(2),
            _ => unreachable!(),
        }
    }

    /// Culls NOP instructions
    pub fn no_operation(&self) -> crate::StaticAnalysis<Self> {
        use crate::Fixup;
        use mos6502::instruction::Instruction;

        Fixup::check(
            !matches!(
                self.opcode(),
                Instruction::NOPA | Instruction::NOP | Instruction::NOPAX
            ),
            "PointlessInstruction",
            Self::skip_opcode,
            0,
        )
    }

    /// Culls instructions that redundantly use a 16-bit address to access zero page. Such
    /// instructions can more optimally be encoded as a zero-page instruction. For example,
    /// `lda $0007, x` could be `lda $07, x`.
    pub fn uses_absolute_mode_unnecessarily(&self) -> crate::StaticAnalysis<Self> {
        use crate::Fixup;
        use mos6502::instruction::Instruction;
        if self.0[2] != 0 {
            return Ok(());
        }
        match self.addressing_mode() {
            AddressingMode::Absolute
                if matches!(
                    self.opcode(),
                    Instruction::NOPA
                        | Instruction::ORA
                        | Instruction::ASL
                        | Instruction::SLO
                        | Instruction::AND
                        | Instruction::ROL
                        | Instruction::RLA
                        | Instruction::EOR
                        | Instruction::LSR
                        | Instruction::SRE
                        | Instruction::ADC
                        | Instruction::ADCnd
                        | Instruction::ROR
                        | Instruction::RRA
                        | Instruction::STY
                        | Instruction::STA
                        | Instruction::STX
                        | Instruction::SAX
                        | Instruction::LDY
                        | Instruction::LDA
                        | Instruction::LDX
                        | Instruction::LAX
                        | Instruction::CPY
                        | Instruction::CPX
                        | Instruction::CMP
                        | Instruction::DEC
                        | Instruction::DCP
                        | Instruction::SBC
                        | Instruction::SBCnd
                        | Instruction::INC
                        | Instruction::ISC
                ) =>
            {
                Fixup::err("Pointless", Self::skip_operand, 0)
            }

            AddressingMode::AbsoluteX
                if matches!(
                    self.opcode(),
                    Instruction::NOPAX
                        | Instruction::ORA
                        | Instruction::SLO
                        | Instruction::ASL
                        | Instruction::AND
                        | Instruction::ROL
                        | Instruction::RLA
                        | Instruction::SRE
                        | Instruction::EOR
                        | Instruction::LSR
                        | Instruction::RRA
                        | Instruction::ADC
                        | Instruction::ADCnd
                        | Instruction::ROR
                        | Instruction::STA
                        | Instruction::LDY
                        | Instruction::LDA
                        | Instruction::DEC
                        | Instruction::DCP
                        | Instruction::CMP
                        | Instruction::ISC
                        | Instruction::SBC
                        | Instruction::SBCnd
                        | Instruction::INC
                ) =>
            {
                Fixup::err("Pointless", Self::skip_operand, 0)
            }

            AddressingMode::AbsoluteY
                if matches!(
                    self.opcode(),
                    Instruction::ADC | Instruction::ADCnd | Instruction::LDX | Instruction::LAX
                ) =>
            {
                Fixup::err("Pointless", Self::skip_operand, 0)
            }
            _ => Ok(()),
        }
    }

    /// Makes the instruction into `rts`
    pub fn make_rts(&self) -> crate::StaticAnalysis<Self> {
        const INSN: u8 = 0x60;
        crate::Fixup::<Self>::check(
            self.0[0] == INSN,
            "DoesNotReturn",
            |i| {
                if i.0[0] <= INSN {
                    i.0[0] = INSN;
                    Ok(())
                } else {
                    Err(crate::StepError::End)
                }
            },
            0,
        )
    }
}

impl<V: mos6502::Variant> crate::Instruction for Instruction<V> {
    fn random() -> Self {
        let mut s = Self(
            [rand::random(), rand::random(), rand::random()],
            Default::default(),
        );
        while !s.valid_opcode() {
            s.0[0] = rand::random();
        }
        s
    }
    fn first() -> Self {
        Self([0, 0, 0], Default::default())
    }
    fn mutate(&mut self) {
        if rand::random() {
            // pick a different opcode
            self.0[0] = rand::random();
            while !self.valid_opcode() {
                self.0[0] = rand::random();
            }
        } else {
            // pick a different operand
            self.0[1] = rand::random();
            self.0[2] = rand::random();
        }
    }
    fn increment(&mut self) -> IterationResult {
        self.incr_at_offset(self.len() - 1)
    }
    fn to_bytes(&self) -> Vec<u8> {
        match self.len() {
            1 => vec![self.0[0]],
            2 => vec![self.0[0], self.0[1]],
            3 => vec![self.0[0], self.0[1], self.0[2]],
            _ => unreachable!("there are no {} length 6502 instructions", self.len()),
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut insn = Self::first();
        insn.0[0] = *bytes.first()?;
        match insn.len() {
            1 => {}
            2 => {
                insn.0[1] = *bytes.get(1)?;
            }
            3 => {
                insn.0[1] = *bytes.get(1)?;
                insn.0[2] = *bytes.get(2)?;
            }
            _ => unreachable!(),
        }
        Some(insn)
    }
}

impl<V: mos6502::Variant> std::fmt::Display for Instruction<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use mos6502::instruction::Instruction;

        match self.opcode() {
            Instruction::ADC => write!(f, "adc"),
            Instruction::ADCnd => write!(f, "adc"),
            Instruction::ALR => write!(f, "alr"),
            Instruction::ANC => write!(f, "anc"),
            Instruction::AND => write!(f, "and"),
            Instruction::ASL => write!(f, "asl"),
            Instruction::ARR => write!(f, "arr"),
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
            Instruction::DCP => write!(f, "dcp"),
            Instruction::DEC => write!(f, "dec"),
            Instruction::DEX => write!(f, "dex"),
            Instruction::DEY => write!(f, "dey"),
            Instruction::EOR => write!(f, "eor"),
            Instruction::INC => write!(f, "inc"),
            Instruction::INX => write!(f, "inx"),
            Instruction::INY => write!(f, "iny"),
            Instruction::ISC => write!(f, "isc"),
            Instruction::JAM => write!(f, "jam"),
            Instruction::JMP => write!(f, "jmp"),
            Instruction::JSR => write!(f, "jsr"),
            Instruction::LAS => write!(f, "las"),
            Instruction::LAX => write!(f, "lax"),
            Instruction::LDA => write!(f, "lda"),
            Instruction::LDX => write!(f, "ldx"),
            Instruction::LDY => write!(f, "ldy"),
            Instruction::LSR => write!(f, "lsr"),
            Instruction::NOP => write!(f, "nop"),
            Instruction::NOPI => write!(f, "nop"),
            Instruction::NOPZ => write!(f, "nop"),
            Instruction::NOPZX => write!(f, "nop"),
            Instruction::NOPA => write!(f, "nop"),
            Instruction::NOPAX => write!(f, "nop"),
            Instruction::ORA => write!(f, "ora"),
            Instruction::PHA => write!(f, "pha"),
            Instruction::PHP => write!(f, "php"),
            Instruction::PHX => write!(f, "phx"),
            Instruction::PHY => write!(f, "phy"),
            Instruction::PLA => write!(f, "pla"),
            Instruction::PLP => write!(f, "plp"),
            Instruction::PLX => write!(f, "plx"),
            Instruction::PLY => write!(f, "ply"),
            Instruction::RLA => write!(f, "rla"),
            Instruction::ROL => write!(f, "rol"),
            Instruction::ROR => write!(f, "ror"),
            Instruction::RRA => write!(f, "rra"),
            Instruction::RTI => write!(f, "rti"),
            Instruction::RTS => write!(f, "rts"),
            Instruction::SAX => write!(f, "sax"),
            Instruction::SBC => write!(f, "sbc"),
            Instruction::SBCnd => write!(f, "sbc"),
            Instruction::SBX => write!(f, "sbx"),
            Instruction::SEC => write!(f, "sec"),
            Instruction::SED => write!(f, "sed"),
            Instruction::SEI => write!(f, "sei"),
            Instruction::SLO => write!(f, "slo"),
            Instruction::SRE => write!(f, "sre"),
            Instruction::STA => write!(f, "sta"),
            Instruction::STP => write!(f, "stp"),
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
            Instruction::USBC => write!(f, "sbc"),
            Instruction::WAI => write!(f, "wai"),
            Instruction::XAA => write!(f, "xaa"),
        }?;
        match self.addressing_mode() {
            AddressingMode::Accumulator => write!(f, " a"),
            AddressingMode::Implied => write!(f, ""),
            AddressingMode::Immediate => write!(f, " #${:02x}", self.0[1]),
            AddressingMode::ZeroPage => write!(f, " ${:02x}", self.0[1]),
            AddressingMode::ZeroPageX => write!(f, " ${:02x}, x", self.0[1]),
            AddressingMode::ZeroPageY => write!(f, " ${:02x}, y", self.0[1]),
            AddressingMode::Relative => write!(f, " {}", self.0[1] as i8),
            AddressingMode::IndexedIndirectX => write!(f, " (${:02x},x)", self.0[1]),
            AddressingMode::IndirectIndexedY => write!(f, " (${:02x}),y", self.0[1]),
            AddressingMode::Absolute => {
                write!(f, " ${:02x}{:02x}", self.0[2], self.0[1])
            }
            AddressingMode::AbsoluteX => {
                write!(f, " ${:02x}{:02x}, x", self.0[2], self.0[1])
            }
            AddressingMode::AbsoluteY => {
                write!(f, " ${:02x}{:02x}, y", self.0[2], self.0[1])
            }
            AddressingMode::Indirect | mos6502::instruction::AddressingMode::BuggyIndirect => {
                write!(f, " (${:02x}{:02x})", self.0[2], self.0[1])
            }
            AddressingMode::ZeroPageIndirect => {
                write!(f, " (${:02x})", self.0[1])
            }
            AddressingMode::AbsoluteIndexedIndirect => {
                write!(f, " (${:02x}{:02x}, x)", self.0[2], self.0[1])
            }
        }?;
        Ok(())
    }
}

impl<V: mos6502::Variant> std::fmt::Debug for Instruction<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Instruction;
        let bytes = self
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<String>>()
            .join(" ");

        let n = format!("{self}");
        write!(f, "{n:15} ; {bytes}")
    }
}
