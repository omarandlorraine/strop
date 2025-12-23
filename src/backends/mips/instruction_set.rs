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
        write!(f, "{:<20}; 0x{:08x}", dasm, self.0)?;
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
        use rand::Rng;

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
        if self.0 >= 0xefff_ffff {
            // There are no valid instructions in this range.
            Err(crate::StepError::End)
        } else {
            self.0 += 1;
            self.fixup()?;
            Ok(())
        }
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

    /// Returns the fixup that skips redundant encoding
    fn pointless(&self) -> crate::StaticAnalysis<Self> {
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
            // addiu is not included here because `addiu something something 0x0` is an alias for
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

}

impl Instruction {
    fn decode(&self) -> trapezoid_core::cpu::Instruction {
        trapezoid_core::cpu::Instruction::from_u32(self.0, 0)
    }

    /// Skip to the next opcode (this increments either the `funct` field or the `opcode` field as
    /// appropriate)
    pub fn next_opcode(&mut self) -> crate::IterationResult {
        if self.0 >= 0xefff_ffff {
            Err(crate::StepError::End)
        } else if self.r() {
            // It's an R format instruction so to go to the next opcode we need to increment by 1
            self.0 += 1;
            self.fixup()
        } else {
            // It's an I or J format instruction. To go to the next opcode, add 0x0400_0000.
            self.0 += 0x0400_0000;
            self.fixup()
        }
    }

    /// Changes the registers which an instruction refers to.
    fn next_registers(&mut self) -> crate::IterationResult {
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
                    | Opcode::Bgtz
                    | Opcode::Syscall
                    | Opcode::Break
            ),
            "InappriopriatelyPlacedControlFlowInstruction",
            Self::next_opcode,
            0,
        )
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

    /// Called after a mutation; this ensures that the u32 member encodes an actually valid MIPS
    /// instruction
    fn fixup(&mut self) -> crate::IterationResult {
        use trapezoid_core::cpu::Opcode;
        use trapezoid_core::cpu::RegisterType;

        fn cop0readable(reg: u8) -> bool {
            match reg {
                // Reading from some coprocessor registers works okay.
                6 => true,  // JMP_DEST
                7 => true,  // DCIC
                8 => true,  // BAD_VADDR
                12 => true, // SR
                13 => true, // CAUSE
                14 => true, // EPC
                15 => true, // PRID

                // Some COP0 registers read garbage; I guess those instructions are
                // pointless
                16..=31 => false,

                // Reading from other coprocessor registers seems to crash the emulator, so
                // we need to exclude the instructions from being generated
                _ => false,
            }
        }
        loop {
            let opcode = self.decode().opcode;

            if matches!(
                opcode,
                Opcode::Invalid | Opcode::Add | Opcode::Sub | Opcode::Addi
            ) {
                // Some instructions do not encode a valid opcode, skip them.
                // And also skip the `add` and `sub` instructions. Because they're worse
                // equivalents to `addu` and `subu`.
                self.next_opcode()?;
                continue;
            }

            if let Opcode::Swc(coprocessor) = opcode {
                if coprocessor == 0 {
                    // COP0; the MIPS exception handling coprocessor thing
                    if !cop0readable(self.decode().rt() as u8) {
                        self.0 = self.0.checked_add(1).unwrap();
                        continue;
                    }
                } else if coprocessor == 2 {
                    // COP2; the Playstation 1 Geometry Transform thing
                } else {
                    // unknown coprocessor; the emulator does not implement it, don't generate
                    // these instructions.
                    self.next_opcode()?;
                    continue;
                }
            }

            if matches!(self.0, 0x0401fffe | 0x1800fffe) {
                // these istructions are relative branches to themselves. Incidentally these seem
                // to crash the emulator
                self.next_opcode()?;
            }

            if let Opcode::Mfc(coprocessor) = opcode {
                if coprocessor == 0 {
                    // COP0; the MIPS exception handling coprocessor thing
                    if !cop0readable(self.decode().rd() as u8) {
                        self.0 = self.0.checked_add(1).unwrap();
                        continue;
                    }
                } else if coprocessor == 2 {
                    // COP2; the Playstation 1 Geometry Transform thing
                } else {
                    // unknown coprocessor; the emulator does not implement it, don't generate
                    // these instructions.
                    self.next_opcode()?;
                    continue;
                }
            }
            if let Opcode::Lwc(coprocessor) = opcode {
                if coprocessor == 0 {
                    // COP0; the MIPS exception handling coprocessor thing
                    match self.decode().rt() as u8 {
                        // writing some coprocessor registers works okay.
                        3 => {}  // BPC
                        5 => {}  // BDA
                        7 => {}  // DCIC
                        9 => {}  // BDAM
                        11 => {} // BPCM
                        12 => {} // SR
                        13 => {} // CAUSE

                        // writing to other coprocessor registers seems to crash the emulator, so
                        // we need to exclude the instructions from being generated
                        _ => {
                            self.0 = self.0.checked_add(1).unwrap();
                            continue;
                        }
                    }
                } else if coprocessor == 2 {
                    // COP2; the Playstation 1 Geometry Transform thing
                } else {
                    // unknown coprocessor; the emulator does not implement it, don't generate
                    // these instructions.
                    self.next_opcode()?;
                    continue;
                }
            }
            if self.r() {
                let rs = self.decode().rs();
                let rt = self.decode().rt();
                let rd = self.decode().rd();
                let shamt = self.decode().imm5();

                if rt != RegisterType::Zero {
                    // Some instructions ignore rt, so skip those opcodes if $rt isn't $zero.
                    if self.read_rt().is_none() && self.write_rt().is_none() {
                        self.next_opcode()?;
                        continue;
                    }
                }

                if rd != RegisterType::Zero {
                    // Some instructions ignore rd, so skip those opcodes if $rd isn't $zero.
                    if self.rd().is_none() {
                        self.next_opcode()?;
                        continue;
                    }
                }

                if rs != RegisterType::Zero {
                    // Some instructions ignore rs, so skip those opcodes if $rs isn't $zero.
                    if self.rs().is_none() {
                        self.next_opcode()?;
                        continue;
                    }
                }

                if shamt != 0 {
                    // Some instructions ignore shamt, so skip those opcodes if the `shamt` bitfield isn't 0.
                    if self.shamt().is_none() {
                        self.next_opcode()?;
                        continue;
                    }
                }
            }
            break Ok(());
        }
    }

    /// Returns the `rt` if the instruction actually reads from the `rt`
    pub fn read_rt(&self) -> Option<RegisterType> {
        use trapezoid_core::cpu::Opcode;
        if matches!(
            self.decode().opcode,
            Opcode::Jr
                | Opcode::Jalr
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
