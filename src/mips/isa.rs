//! Module representing MIPS I instruction set architecture

use crate::dataflow::DataFlow;
use crate::Encode;
use crate::Step;
use trapezoid_core::cpu::Instruction;
use trapezoid_core::cpu::RegisterType;

/// Represents a MIPS instruction
#[derive(Clone, PartialEq)]
pub struct Insn(u32);

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.decode())
    }
}

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}\t; 0x{:08x}", self.decode(), self.0)
    }
}

impl crate::subroutine::ShouldReturn for Insn {
    fn allowed_in_subroutine(&self) -> Result<(), crate::StaticAnalysis<Self>> {
        use trapezoid_core::cpu::Opcode;
        let decoded = self.decode();

        match decoded.opcode {
            Opcode::J | Opcode::Jal => Err(crate::StaticAnalysis::<Self> {
                advance: Self::next_opcode,
                offset: 0,
                reason: "OpcodeNotAllowedInSubroutines",
            }),
            _ => Ok(()),
        }
    }

    fn should_return(&self, offset: usize) -> Result<(), crate::StaticAnalysis<Self>> {
        if *self == Self::jr_ra() {
            return Ok(());
        }
        Err(crate::StaticAnalysis::<Self> {
            advance: Self::make_return,
            offset,
            reason: "ShouldReturn",
        })
    }
}

impl crate::Branch for Insn {
    fn offset(&self) -> Option<isize> {
        use trapezoid_core::cpu::Opcode;
        let decoded = self.decode();

        match decoded.opcode {
            Opcode::Beq
            | Opcode::Bne
            | Opcode::Bgtz
            | Opcode::Blez
            | Opcode::Bltz
            | Opcode::Bgez
            | Opcode::Bltzal
            | Opcode::Bgezal => Some(decoded.imm16() as i16 as isize),
            _ => None,
        }
    }

    fn branch_fixup(&self, permissibles: &[isize]) -> Result<(), crate::StaticAnalysis<Self>> {
        let Some(offset) = self.offset() else {
            return Ok(());
        };
        if !permissibles.contains(&offset) {
            Err(crate::StaticAnalysis::<Self> {
                advance: Self::next,
                offset: 0,
                reason: "BackwardBranchNotInRange",
            })
        } else {
            // backward branch in range
            Ok(())
        }
    }
}

impl Insn {
    fn decode(&self) -> Instruction {
        Instruction::from_u32(self.0, 0)
    }

    /// Returns a `jr $ra` instruction, which is what's used to return from subroutines.
    pub fn jr_ra() -> Self {
        Self(0x03E00008)
    }

    /// Returns true if the instruction is an `R` format instruction
    pub fn r(&self) -> bool {
        self.0 & 0xfc000000 == 0
    }

    /// Returns true if the instruction is an `J` format instruction
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

    fn make_return(&mut self) -> crate::IterationResult {
        // TODO: There are other possible return instructions here.
        use std::cmp::Ordering;

        match self.0.cmp(&Self::jr_ra().0) {
            Ordering::Less => {
                *self = Self::jr_ra();
                Ok(())
            }
            Ordering::Greater => Err(crate::StepError::End),
            Ordering::Equal => unreachable!(),
        }
    }

    /// Changes the registers which an instruction refers to.
    fn next_registers(&mut self) -> crate::IterationResult {
        use crate::Step;
        if self.r() {
            // R format instruction: mask off the shamt and func fields, and then increment.
            self.0 |= 0x7ff;
            self.next()
        } else if self.i() {
            // I format instruction: mask off the imm field, and then increment.
            self.0 |= 0xffff;
            self.next()?;
            Ok(())
        } else {
            // J format instruction: this shouldn't really even be reachable.
            self.next_opcode()
        }
    }

    /// Called after a mutation; this ensures that the u32 member encodes an actually valid MIPS
    /// instruction
    fn fixup(&mut self) -> crate::IterationResult {
        use trapezoid_core::cpu::Opcode;
        use trapezoid_core::cpu::RegisterType;

        loop {
            let opcode = self.decode().opcode;

            if opcode == Opcode::Invalid {
                // Some instructions do not encode a valid opcode, skip these.
                self.next_opcode()?;
                continue;
            }

            if let Opcode::Mfc(coprocessor) = opcode {
                if coprocessor == 0 {
                    // COP0; the MIPS exception handling coprocessor thing
                    match self.decode().rd() as u8 {
                        // Reading from some coprocessor registers works okay.
                        6 => {}  // JMP_DEST
                        7 => {}  // DCIC
                        8 => {}  // BAD_VADDR
                        12 => {} // SR
                        13 => {} // CAUSE
                        14 => {} // EPC
                        15 => {} // PRID

                        // Some COP0 registers read garbage; I guess those instructions are
                        // pointless
                        16..=31 => self.next()?,

                        // Reading from other coprocessor registers seems to crash the emulator, so
                        // we need to exclude the instructions from being generated
                        _ => self.next()?,
                    }
                } else if coprocessor == 2 {
                    // COP2; the Playstation 1 Geometry Transform thing
                } else {
                    // unknown coprocessor; the emulator does not implement it, don't generate
                    // these instructions.
                    self.next_opcode()?;
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

    /// true if the instruction reads from register $zero.
    fn reads_or_writes_zero(&self) -> bool {
        use trapezoid_core::cpu::RegisterType;
        self.reads(&RegisterType::Zero) | self.writes(&RegisterType::Zero)
    }

    /// true if the instruction reads from two registers but $rs and $rt are not in canonical
    /// order. (This is useful for culling the search space because A+B is the same as B+A. But the
    /// bruteforce search only needs to visit one of these.)
    fn commutative_reads_out_of_order(&self) -> bool {
        if self.rs().is_none() {
            return false;
        };

        if self.read_rt().is_none() {
            return false;
        };

        let rt_raw = (self.0 >> 16) as u8 & 0x1f;
        let rs_raw = (self.0 >> 21) as u8 & 0x1f;
        rt_raw < rs_raw
    }

    /// Returns true if the immediate value is Zero;
    fn immediate_zero(&self) -> bool {
        self.0 & 0x0000ffff == 0x00000000
    }

    /// Returns true iff I have deemed the instruction to be a pointless one
    pub fn pointless(&self) -> bool {
        use trapezoid_core::cpu::Opcode;
        use trapezoid_core::cpu::RegisterType;
        let d = self.decode();

        match d.opcode {
            // Shift instructions shouldn't need to shift by zero.
            Opcode::Srl | Opcode::Sra | Opcode::Sll => d.imm5() == 0,
            Opcode::Sllv | Opcode::Srav | Opcode::Srlv => d.rs() == RegisterType::Zero,

            // Arithmetic instructions don't need to read from $zero. (The exception is `or`; `or`
            // with $zero is the idiomatic way to copy a value from one register to another.

            // Also some of these operations are commutative; let's make sure that they don't read
            // from two registers in the "wrong" order.
            Opcode::Add => self.commutative_reads_out_of_order() | self.reads_or_writes_zero(),
            Opcode::Or => self.commutative_reads_out_of_order(),
            Opcode::Addu => self.commutative_reads_out_of_order() | self.reads_or_writes_zero(),
            Opcode::Xor => self.commutative_reads_out_of_order() | self.reads_or_writes_zero(),
            Opcode::And => self.commutative_reads_out_of_order() | self.reads_or_writes_zero(),
            Opcode::Sub => self.reads_or_writes_zero(),
            Opcode::Subu => self.reads_or_writes_zero(),

            // Some immediate values shouldn't be 0x0000
            Opcode::Addi => self.immediate_zero(),
            Opcode::Addiu => self.immediate_zero(),
            Opcode::Ori => self.immediate_zero(),
            Opcode::Xori => self.immediate_zero(),

            // Why would you want to generate a sequence that has NOPs in it
            Opcode::Nop => true,
            _ => false,
        }
    }
}

impl crate::Disassemble for Insn {
    fn dasm(&self) {
        println!("\t{:?}", self);
    }
}

impl Step for Insn {
    fn first() -> Self {
        Self(0)
    }

    fn next(&mut self) -> crate::IterationResult {
        if self.0 >= 0xefff_ffff {
            // There are no valid instructions in this range.
            Err(crate::StepError::End)
        } else {
            self.0 += 1;
            self.fixup()?;
            Ok(())
        }
    }
}

impl Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl DataFlow<RegisterType> for Insn {
    fn reads(&self, datum: &RegisterType) -> bool {
        Some(datum) == self.rs().as_ref() || Some(datum) == self.read_rt().as_ref()
    }

    fn writes(&self, datum: &RegisterType) -> bool {
        Some(datum) == self.rd().as_ref() || Some(datum) == self.write_rt().as_ref()
    }

    fn sa(&self) -> crate::StaticAnalysis<Self> {
        crate::StaticAnalysis::<Self> {
            advance: Self::next_registers,
            offset: 0,
            reason: "Dataflow",
        }
    }
}

#[cfg(test)]
mod test {

    fn check_instruction(insn: &super::Insn) {
        // Make sure the instruction doesn't think it's both reading from and writing to $rt.
        if insn.read_rt().is_some() {
            assert!(insn.write_rt().is_none(), "check_instruction(&Insn(0x{:08x})); // {insn} seems to both read and write for $rt.", insn.0);
        }

        // Make sure the disassembly doesn't just give us a hexadecimal value.
        assert!(
            u32::from_str_radix(&format!("{insn}"), 16).is_err(),
            "check_instruction(&Insn(0x{:08x})); // disassembly missing.",
            insn.0
        );

        // If the disassembly contains the substring "a2", then the instruction needs to report
        // that it reads/writes that register.
        if format!("{}", insn).contains("a2") {
            use trapezoid_core::cpu::RegisterType;
            match (insn.rs(), insn.rd(), insn.read_rt(), insn.write_rt()) {
                (Some(RegisterType::A2), _, _, _) => {}
                (_, Some(RegisterType::A2), _, _) => {}
                (_, _, Some(RegisterType::A2), _) => {}
                (_, _, _, Some(RegisterType::A2)) => {}
                _ => panic!(
                    "check_instruction(&Insn(0x{:08x})); // register checks for \"{insn}\"",
                    insn.0
                ),
            }
        }

        assert_ne!(
            format!("{}", insn),
            "Invalid instruction",
            "check_instruction(&Insn(0x{:08x})); // couldn't disassemble \"{insn}\"",
            insn.0
        );
    }

    #[test]
    fn regressions() {
        use super::Insn;
        check_instruction(&Insn(0x0000001a));
        check_instruction(&Insn(0x04000000));
        check_instruction(&Insn(0x08000000));
        check_instruction(&Insn(0x10060000));
        check_instruction(&Insn(0x14000000));
        check_instruction(&Insn(0x18000000));
        check_instruction(&Insn(0x1c000000));
        check_instruction(&Insn(0x20000000));
        check_instruction(&Insn(0x28000000));
        check_instruction(&Insn(0x30000000));
        check_instruction(&Insn(0x34000000));
        check_instruction(&Insn(0x38000000));
        check_instruction(&Insn(0x3c000000));
        check_instruction(&Insn(0x40000000));
        check_instruction(&Insn(0x80000001));
        check_instruction(&Insn(0x88000000));
        check_instruction(&Insn(0xa0000000));
        check_instruction(&Insn(0xa8000000));
        check_instruction(&Insn(0xb8000000));
        check_instruction(&Insn(0xc0000000));
    }

    #[test]
    #[ignore]
    fn no_duplicates() {
        use super::Insn;
        use crate::Step;

        // because of dont-care fields in some of the instructions, the same instruction may have
        // two encodings. I consider it an error to use the "wrong one".
        //
        // For example, the `and` opcode ignores the five bit `shamt` field, so there's 32 possible
        // encodings for every `and` instruction. This is going to have an obvious negative impact
        // on the bruteforce search.
        //
        // So this test finds these guys.

        let mut i = Insn::first();

        while i.next().is_ok() {
            println!("Checking for duplicates of {i}");
            let mut j = i.clone();
            while j.next().is_ok() {
                if format!("{i}") == format!("{j}") {
                    if i.decode().rd() != j.decode().rd() {
                        println!("they differ in $rd");
                    }
                    if i.decode().rt() != j.decode().rt() {
                        println!("they differ in $rt");
                    }
                    if i.decode().rs() != j.decode().rs() {
                        println!("they differ in $rs");
                    }
                    if i.decode().imm5() != j.decode().imm5() {
                        println!("they differ in $shamt");
                    }
                    panic!(
                        "These two instructions both encode {i}: 0x{:08x} 0x{:08x}",
                        i.0, j.0
                    );
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn can_iterate_over_all_instructions() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn::first();

        while i.next().is_ok() {
            check_instruction(&i);
        }
    }

    #[test]
    fn can_iterate_over_the_first_few_instructions() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn::first();

        for _ in 0..0xffff {
            assert!(i.next().is_ok());
            check_instruction(&i);
        }
    }

    #[test]
    fn can_iterate_until_the_end() {
        use super::Insn;
        use crate::Step;

        let mut i = Insn(0xefff_ff00);

        while i.next().is_ok() {
            check_instruction(&i);
        }
    }

    #[test]
    fn jr_ra() {
        use super::Insn;
        use trapezoid_core::cpu::Opcode;
        use trapezoid_core::cpu::RegisterType;

        let i = Insn::jr_ra();
        let d = i.decode();
        assert_eq!(d.opcode, Opcode::Jr);
        assert_eq!(d.imm5(), 0);
        assert_eq!(d.rt(), RegisterType::Zero);
        assert_eq!(d.rd(), RegisterType::Zero);
        assert_eq!(d.rs(), RegisterType::Ra);
    }
}
