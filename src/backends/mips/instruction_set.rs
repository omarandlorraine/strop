use crate::{Fixup, StaticAnalysis};
use trapezoid_core::cpu::RegisterType;

/// Represents a MIPS instruction
#[derive(Clone, PartialEq)]
pub struct Instruction(u32);

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.decode())
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let dasm = format!("{}", self.decode());
        write!(f, "{:<25}; 0x{:08x}", dasm, self.0)?;
        if let Some(rd) = self.rd() {
            write!(f, " $rd={rd:?}")?;
        }
        if let Some(rt) = self.read_rt() {
            write!(f, " $rt={rt:?}")?;
        }
        if let Some(rt) = self.write_rt() {
            write!(f, " $rt={rt:?}")?;
        }
        if let Some(rs) = self.rs() {
            write!(f, " $rs={rs:?}")?;
        }
        if let Some(shamt) = self.shamt() {
            write!(f, " $shamt={shamt:?}")?;
        }
        if let Some(imm) = self.imm() {
            write!(f, " $imm={imm:?}")?;
        }
        Ok(())
    }
}

impl crate::dataflow::DataFlow<trapezoid_core::cpu::RegisterType> for Instruction {
    fn reads(&self, datum: &trapezoid_core::cpu::RegisterType) -> bool {
        Some(datum) == self.rs().as_ref()
            || Some(datum) == self.read_rt().as_ref()
            || (*datum == trapezoid_core::cpu::RegisterType::Hi
                && self.decode().opcode == trapezoid_core::cpu::Opcode::Mfhi)
            || (*datum == trapezoid_core::cpu::RegisterType::Lo
                && self.decode().opcode == trapezoid_core::cpu::Opcode::Mflo)
    }

    fn writes(&self, datum: &trapezoid_core::cpu::RegisterType) -> bool {
        use trapezoid_core::cpu::{Opcode, RegisterType};
        if matches!(datum, RegisterType::Hi) {
            matches!(
                self.decode().opcode,
                Opcode::Mult | Opcode::Multu | Opcode::Div | Opcode::Divu | Opcode::Mthi
            )
        } else if matches!(datum, RegisterType::Lo) {
            matches!(
                self.decode().opcode,
                Opcode::Mult | Opcode::Multu | Opcode::Div | Opcode::Divu | Opcode::Mtlo
            )
        } else {
            Some(datum) == self.rd().as_ref() || Some(datum) == self.write_rt().as_ref()
        }
    }

    fn sa(&self, offset: usize) -> crate::Fixup<Self> {
        crate::Fixup::new("Dataflow", Self::next_registers, offset)
    }
}

impl crate::Instruction for Instruction {
    fn random() -> Self {
        Self(rand::random())
    }
    fn first() -> Self {
        Self(0)
    }
    fn mutate(&mut self) {
        use rand::RngExt;

        loop {
            if rand::random() {
                // could flip a bit in the instruction word
                let mask: u32 = 1 << rand::rng().random_range(0..32);
                self.0 ^= mask;
            } else {
                // could completely change the instruction word to something completely different
                self.0 = rand::random()
            }
            if self.fixup().is_ok() {
                break;
            }
        }
    }
    fn increment(&mut self) -> crate::IterationResult {
        self.inner_increment()?;
        self.fixup()
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into()
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self(u32::from_le_bytes([
            *bytes.first()?,
            *bytes.get(1)?,
            *bytes.get(2)?,
            *bytes.get(3)?,
        ])))
    }
}

impl Instruction {
    /// Decodes the instruction
    pub fn decode(&self) -> trapezoid_core::cpu::Instruction {
        trapezoid_core::cpu::Instruction::from_u32(self.0, 0)
    }

    fn inner_increment(&mut self) -> crate::IterationResult {
        if self.0 >= 0xefff_ffff {
            // There are no valid instructions in this range.
            Err(crate::StepError::End)
        } else {
            self.0 += 1;
            Ok(())
        }
    }
    fn inner_next_opcode(&mut self) -> crate::IterationResult {
        if self.0 >= 0xefff_ffff {
            Err(crate::StepError::End)
        } else if self.r() {
            // It's an R format instruction so to go to the next opcode we need to increment by 1
            self.0 += 1;
            Ok(())
        } else {
            // It's an I or J format instruction. To go to the next opcode, add 0x0400_0000.
            self.0 += 0x0400_0000;
            Ok(())
        }
    }

    /// Skip to the next opcode (this increments either the `funct` field or the `opcode` field as
    /// appropriate)
    pub fn next_opcode(&mut self) -> crate::IterationResult {
        self.inner_next_opcode()?;
        self.fixup()
    }

    /// Changes the registers which an instruction refers to.
    pub fn next_registers(&mut self) -> crate::IterationResult {
        use crate::Instruction;
        if self.r() {
            // R format instruction: mask off the shamt and func fields, and then increment.
            self.0 |= 0x7ff;
            self.increment()
        } else if self.i() {
            // I format instruction: mask off the imm field, and then increment.
            self.0 |= 0xffff;
            self.increment()?;
            Ok(())
        } else {
            // J format instruction: this shouldn't really even be reachable.
            self.next_opcode()
        }
    }

    /// Returns the fixup that makes this a `jr $ra` instruction
    pub fn make_jr_ra(&self) -> crate::StaticAnalysis<Self> {
        const INSN: u32 = 0x03e00008;
        crate::Fixup::<Self>::check(
            self.0 == INSN,
            "DoesNotReturn",
            |i| {
                if i.0 <= INSN {
                    i.0 = INSN;
                    Ok(())
                } else {
                    Err(crate::StepError::End)
                }
            },
            0,
        )
    }

    fn redundant_encoding(&self) -> crate::StaticAnalysis<Self> {
        use crate::Instruction;
        Err(crate::Fixup::<Self> {
            advance: |i| {
                if i.r() {
                    i.next_opcode()
                } else {
                    i.increment()
                }
            },
            offset: 0,
            reason: "RedundantEncoding",
        })
    }

    /// Returns true if the instruction is a relative branch
    pub fn is_relative_branch(&self) -> bool {
        use trapezoid_core::cpu::Opcode;
        matches!(
            self.decode().opcode,
            Opcode::Bgez
                | Opcode::Beq
                | Opcode::Bltz
                | Opcode::Bne
                | Opcode::Blez
                | Opcode::Bgtz
                | Opcode::Bltzal
                | Opcode::Bgezal
        )
    }

    /// Returns the fixup that skips impure instructions
    pub fn make_pure(&self) -> crate::StaticAnalysis<Self> {
        use trapezoid_core::cpu::Opcode;

        crate::Fixup::<Self>::check(
            !matches!(
                self.decode().opcode,
                Opcode::Lh
                    | Opcode::Lw
                    | Opcode::Lb
                    | Opcode::Lhu
                    | Opcode::Lwr
                    | Opcode::Lbu
                    | Opcode::Lwl
            ),
            "Impure",
            Self::next_opcode,
            0,
        )
    }

    /// Returns the fixup that skips jump instructions and branches and anything else that can
    /// terminate a basic block
    pub fn make_not_control_flow(&self) -> crate::StaticAnalysis<Self> {
        use trapezoid_core::cpu::Opcode;

        crate::Fixup::<Self>::check(
            !matches!(
                self.decode().opcode,
                Opcode::Jalr
                    | Opcode::Jal
                    | Opcode::J
                    | Opcode::Jr
                    | Opcode::Bgez
                    | Opcode::Beq
                    | Opcode::Bltz
                    | Opcode::Bne
                    | Opcode::Blez
                    | Opcode::Bltzal
                    | Opcode::Bgezal
                    | Opcode::Bgtz
                    | Opcode::Syscall
                    | Opcode::Break
            ),
            "InappriopriatelyPlacedControlFlowInstruction",
            Self::next_opcode,
            0,
        )
    }

    /// Returns the fixup that skips redundant encoding
    pub fn make_not_redundantly_encoded(&self) -> crate::StaticAnalysis<Self> {
        use trapezoid_core::cpu::Opcode;

        // Writing to $zero is pointless
        if self.rd() == Some(RegisterType::Zero) {
            return self.redundant_encoding();
        }
        if self.write_rt() == Some(RegisterType::Zero) {
            return self.redundant_encoding();
        }

        if matches!(self.decode().opcode, Opcode::And | Opcode::Or) {
            // if the two read operands are the same then this is another move instruction
            if self.rs() == self.rs() {
                return self.redundant_encoding();
            }
        }

        if matches!(
            self.decode().opcode,
            Opcode::Addi | Opcode::Ori | Opcode::Xori | Opcode::Andi
        ) {
            // adding/subtracting zero is pointless
            // addiu is not included here because 1addiu something something 0x0` is an alias for
            // move
            if self.imm() == Some(0) {
                return self.redundant_encoding();
            }
        }

        if matches!(
            self.decode().opcode,
            Opcode::And
                | Opcode::Nor
                | Opcode::Or
                | Opcode::Xor
                | Opcode::Add
                | Opcode::Addu
                | Opcode::Sub
                | Opcode::Subu
        ) {
            // if one of the operands is zero then this is pointless
            if self.rs() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }
            if self.read_rt() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }
        }

        if matches!(
            self.decode().opcode,
            Opcode::Srl | Opcode::Sra | Opcode::Slt | Opcode::Sltu | Opcode::Sll
        ) {
            // Shift instructions.

            // Zero shifted over is just zero, so this may as well be a straight move
            if self.read_rt() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }

            // Shifting by zero is equivalent to a straight move
            if self.shamt() == Some(0) {
                return self.redundant_encoding();
            }
        }

        if matches!(
            self.decode().opcode,
            Opcode::Sllv | Opcode::Srav | Opcode::Srlv
        ) {
            // Shift instructions.

            // Shifting by zero is equivalent to a straight move
            if self.rs() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }
        }

        if matches!(
            self.decode().opcode,
            Opcode::Mult | Opcode::Multu | Opcode::Div | Opcode::Divu
        ) {
            // when multiplying or dividing, it's stupid for either operand to be $zero
            if self.read_rt() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }

            if self.rs() == Some(RegisterType::Zero) {
                return self.redundant_encoding();
            }
        }
        Ok(())
    }

    /// Returns true if the instruction is an `R` format instruction
    pub fn r(&self) -> bool {
        self.0 & 0xfc000000 == 0
    }

    /// Returns true if the instruction is an `I` format instruction
    pub fn i(&self) -> bool {
        !self.r() && !self.j()
    }

    /// Returns true if the instruction is an `J` format instruction
    pub fn j(&self) -> bool {
        use trapezoid_core::cpu::Opcode;
        if self.r() {
            false
        } else {
            matches!(self.decode().opcode, Opcode::J | Opcode::Jal)
        }
    }

    /// Some values for the opcode field are not valid; skip these.
    pub fn valid_opcode(&self) -> StaticAnalysis<Instruction> {
        Fixup::<Self>::check(
            !matches!(self.decode().opcode, trapezoid_core::cpu::Opcode::Invalid),
            "InvalidOpcode",
            Self::inner_next_opcode,
            0,
        )
    }

    /// Excludes instructions that tickle coprocessors not known to the emulator. Also checks that
    /// instructions writing to and read from COP0 only use valid COP0 registers.
    pub fn coprocessor_known(&self) -> StaticAnalysis<Instruction> {
        use trapezoid_core::cpu::Opcode;

        fn check_cop0r(rd: u8) -> StaticAnalysis<Instruction> {
            // not all the registers in COP0 are backed by hardware to anything useful
            // so make sure the instruction doesn't try to access anything that's not there
            Fixup::<Instruction>::check(
                matches!(
                    rd,
                    6 | // JMP_DEST
                    7 | // DCIC
                    8 | // BAD_VADDR
                    12| // SR
                    13| // CAUSE
                    14| // EPC
                    15 // PRID
                ),
                "InvalidCop0RegisterNumber",
                Instruction::inner_increment,
                0,
            )
        }
        fn check_cop0w(rd: u8) -> StaticAnalysis<Instruction> {
            // not all the registers in COP0 are backed by hardware to anything useful
            // so make sure the instruction doesn't try to access anything that's not there
            Fixup::<Instruction>::check(
                matches!(
                    rd,
                    3 | // BPC
                    5 | // BDA
                    7 | // DCIC
                    9 | // BDAM
                    11| // BPCM
                    12| // SR
                    13 // CAUSE
                ),
                "InvalidCop0RegisterNumber",
                Instruction::inner_increment,
                0,
            )
        }
        fn check_cop_number(coproc: u8) -> StaticAnalysis<Instruction> {
            Fixup::<Instruction>::check(
                coproc == 2,
                "InvalidCoprocessor",
                Instruction::inner_next_opcode,
                0,
            )
        }

        match self.decode().opcode {
            Opcode::Swc(0) => check_cop0r(self.decode().rt() as u8),
            Opcode::Mfc(0) => check_cop0r(self.decode().rt() as u8),
            Opcode::Lwc(0) => check_cop0w(self.decode().rt() as u8),
            Opcode::Swc(c) => check_cop_number(c),
            Opcode::Lwc(c) => check_cop_number(c),
            _ => Ok(()),
        }
    }

    /// Skip anything not supported by trapezoid-core
    pub fn supported(&self) -> StaticAnalysis<Instruction> {
        self.valid_opcode()?;
        self.coprocessor_known()?;
        Ok(())
    }

    /// Skip add/subtract instruction that throw overflow exceptions
    /// In some contexts, these instructions are just worse versions of the non-exception ones.
    pub fn no_overflow_exceptions(&self) -> StaticAnalysis<Instruction> {
        use trapezoid_core::cpu::Opcode;
        Fixup::<Self>::check(
            !matches!(
                self.decode().opcode,
                Opcode::Sub | Opcode::Addi | Opcode::Add
            ),
            "UnnecessaryOverflowCheck",
            Self::inner_next_opcode,
            0,
        )
    }

    /// Called after a mutation; this ensures that the u32 member encodes an actually valid MIPS
    /// instruction that can be run by the emulator.
    fn fixup(&mut self) -> crate::IterationResult {
        while let Err(fixup) = self.supported() {
            (fixup.advance)(self)?;
        }
        Ok(())
    }

    /// Returns the `rt` if the instruction actually reads from the `rt`
    pub fn read_rt(&self) -> Option<RegisterType> {
        use trapezoid_core::cpu::Opcode;
        if matches!(
            self.decode().opcode,
            Opcode::Jr
                | Opcode::Syscall
                | Opcode::Break
                | Opcode::Mfhi
                | Opcode::Mthi
                | Opcode::Mflo
                | Opcode::Mtlo
                | Opcode::J
                | Opcode::Jal
                | Opcode::Addi
                | Opcode::Addiu
                | Opcode::Slti
                | Opcode::Sltiu
                | Opcode::Andi
                | Opcode::Ori
                | Opcode::Xori
                | Opcode::Lui
                | Opcode::Lb
                | Opcode::Lbu
                | Opcode::Lh
                | Opcode::Lhu
                | Opcode::Lw
                | Opcode::Lwl
                | Opcode::Lwr
                | Opcode::Bgtz
                | Opcode::Blez
                | Opcode::Bltz
                | Opcode::Bgez
                | Opcode::Bltzal
                | Opcode::Bgezal
                | Opcode::Mfc(_)
                | Opcode::Lwc(_)
                | Opcode::Swc(_)
        ) {
            return None;
        }
        Some(self.decode().rt())
    }

    /// Returns the `rt` if the instruction actually writes to the `rt`
    pub fn write_rt(&self) -> Option<RegisterType> {
        use trapezoid_core::cpu::Opcode;
        if matches!(
            &self.decode().opcode,
            Opcode::Srl
                | Opcode::Sra
                | Opcode::Sllv
                | Opcode::Srlv
                | Opcode::Srav
                | Opcode::Jr
                | Opcode::Jalr
                | Opcode::Syscall
                | Opcode::Break
                | Opcode::Mfhi
                | Opcode::Mthi
                | Opcode::Mflo
                | Opcode::Mtlo
                | Opcode::Mult
                | Opcode::Multu
                | Opcode::Div
                | Opcode::Divu
                | Opcode::Add
                | Opcode::Addu
                | Opcode::Sub
                | Opcode::Subu
                | Opcode::And
                | Opcode::Or
                | Opcode::Nor
                | Opcode::Xor
                | Opcode::Slt
                | Opcode::Sltu
                | Opcode::Sll
                | Opcode::Bltz
                | Opcode::Bgez
                | Opcode::Bltzal
                | Opcode::Bgezal
                | Opcode::J
                | Opcode::Jal
                | Opcode::Beq
                | Opcode::Bne
                | Opcode::Blez
                | Opcode::Bgtz
                | Opcode::Sb
                | Opcode::Sh
                | Opcode::Sw
                | Opcode::Swl
                | Opcode::Swr
                | Opcode::Swc(_)
                | Opcode::Lwc(_)
        ) {
            return None;
        }
        Some(self.decode().rt())
    }

    /// Returns the `rd` if the instruction actually writes to `rd`
    pub fn rd(&self) -> Option<RegisterType> {
        use trapezoid_core::cpu::Opcode;
        if !self.r() {
            return None;
        }
        // opcodes which ignore $rd:
        if [
            Opcode::Syscall,
            Opcode::Break,
            Opcode::Mthi,
            Opcode::Mtlo,
            Opcode::Mult,
            Opcode::Multu,
            Opcode::Div,
            Opcode::Divu,
            Opcode::Jr,
        ]
        .contains(&self.decode().opcode)
        {
            return None;
        }
        Some(self.decode().rd())
    }

    /// Returns the `rs` if the instruction actually reads from the `rs`
    pub fn rs(&self) -> Option<RegisterType> {
        use trapezoid_core::cpu::Opcode;
        if [
            Opcode::Srl,
            Opcode::Sra,
            Opcode::Jalr,
            Opcode::Syscall,
            Opcode::Break,
            Opcode::Mfhi,
            Opcode::Mthi,
            Opcode::Mflo,
            Opcode::Sll,
        ]
        .contains(&self.decode().opcode)
        {
            return None;
        }
        Some(self.decode().rs())
    }

    /// Returns the `imm` field for I-format instructions
    pub fn imm(&self) -> Option<u16> {
        if self.i() {
            Some((self.0 & 0xffff).try_into().unwrap())
        } else {
            None
        }
    }

    /// Returns the `shamt` if the instruction actually uses the `shamt`
    pub fn shamt(&self) -> Option<u8> {
        use trapezoid_core::cpu::Opcode;
        if !self.r() {
            // If it's not an R-Type instruction then there's no `shamt` bitfield.
            return None;
        }
        match self.decode().opcode {
            Opcode::Sllv
            | Opcode::Srlv
            | Opcode::Srav
            | Opcode::Jr
            | Opcode::Jalr
            | Opcode::Syscall
            | Opcode::Break
            | Opcode::Mfhi
            | Opcode::Mthi
            | Opcode::Mflo
            | Opcode::Mtlo
            | Opcode::Mult
            | Opcode::Multu
            | Opcode::Div
            | Opcode::Divu
            | Opcode::Add
            | Opcode::Addu
            | Opcode::Sub
            | Opcode::Subu
            | Opcode::And
            | Opcode::Or
            | Opcode::Xor
            | Opcode::Nor
            | Opcode::Slt
            | Opcode::Sltu => None,
            _ => Some(self.decode().imm5()),
        }
    }
}
