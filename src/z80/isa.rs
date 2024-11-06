//! A module for the representation of Z80 machine instructions.

/// Represents a Z80 machine instruction
#[derive(Clone, Copy, PartialOrd, PartialEq, Default)]
pub struct Insn([u8; 5]);

impl crate::Iterable for Insn {
    fn first() -> Self {
        Self([0, 0, 0, 0, 0])
    }

    fn step(&mut self) -> bool {
        use crate::Encode;
        if self.0[0] == 0xff {
            false
        } else {
            self.incr_at_offset(self.len() - 1);
            self.fixup();
            true
        }
    }
}

impl Insn {
    /// constructs a return instruction `ret`
    pub fn ret() -> Self {
        Self::new(&[0xc9])
    }

    /// Returns `true` if the instruction does any kind of flow control, `false` otherwise
    pub fn is_flow_control(&self) -> bool {
        match self.0[0] {
            0x10 => /*djnz*/ true,
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => /* jr */ true,
            0x76 => /* halt */ true,
            0xc0 | 0xc8 | 0xc9 | 0xd0 | 0xd8 | 0xe0 | 0xe8 | 0xf0 | 0xf8 /* ret */ => true,
            0xc2 | 0xd2 | 0xe2 | 0xf2 | 0xc3 | 0xca | 0xda | 0xea | 0xfa | 0xe9 /* jp */ => true,
            0xed => /*reti*/ self.0[1] == 0x4d,
            0xdd => /* jp */ self.0[1] == 0xe9,
            0xfd => /* jp */ self.0[1] == 0xe9,
            _ => false,
        }
    }

    pub fn allowed_in_pure_functions(&self) -> bool {
        match self.0[0] {
            _ => todo!(),
        }
    }

    /// Increments the opcode, and sets all subsequent bytes (i.e. the operand) to 0.
    pub fn next_opcode(&mut self) -> bool {
        if self.0[0] == 0xff {
            false
        } else if self.0[0] == 0xcb && self.0[1] < 0xff {
            self.0[1] += 1;
            self.0[2] = 0;
            self.0[3] = 0;
            self.0[4] = 0;
            self.fixup();
            true
        } else {
            self.0[0] += 1;
            self.0[1] = 0;
            self.0[2] = 0;
            self.0[3] = 0;
            self.0[4] = 0;
            self.fixup();
            true
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.decode().to_bytes()
    }

    fn len(&self) -> usize {
        self.decode().to_bytes().len()
    }
}

impl Insn {
    /// Constructs a new Insn from a slice of bytes
    pub fn new(mc: &[u8]) -> Self {
        let mut enc = [0, 0, 0, 0, 0];
        enc[..mc.len().min(5)].copy_from_slice(mc);
        Self(enc)
    }

    /// Decodes the instruction and returns a `dez80::Instruction`.
    pub fn decode(&self) -> dez80::Instruction {
        let encoding = Vec::<_>::from(self.0);
        let e = dez80::Instruction::decode_one(&mut encoding.as_slice());
        match e {
            Ok(e) => e,
            Err(e) => panic!(
                "couldn't encode {:?}: {:?}",
                self.0
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<String>>()
                    .join(" "),
                e
            ),
        }
    }

    fn incr_at_offset(&mut self, offset: usize) {
        if let Some(nb) = self.0[offset].checked_add(1) {
            self.0[offset] = nb;
        } else {
            self.0[offset] = 0;
            self.incr_at_offset(offset - 1)
        }
    }

    fn fixup(&mut self) {
        if matches!(self.0[0], 0xdd | 0xed) {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [
                0x44, 0x4c, 0x54, 0x5c, 0x60, 0x77, 0x7c, 0x84, 0x8c, 0x94, 0x9c, 0xa4, 0xac, 0xb4,
                0xbc, 0xcb, 0xe1, 0xe3, 0xe5, 0xe9, 0xf9,
            ] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return;
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
            return;
        }

        if self.0[0] == 0xfd {
            // since this is a prefixed instruction, make sure it's an instruction that actually
            // needs the prefix
            for opcode in [0x09] {
                if self.0[1] < opcode {
                    self.0[1] = opcode;
                    return;
                }
            }

            // After this range are instructions which do not need the dd/ed prefix.
            self.0 = [self.0[0] + 1, 0, 0, 0, 0];
        }
    }

    /// Returns true iff the instruction affects the value in the register specified by `f`
    pub fn produces(&self, fact: crate::z80::dataflow::Fact) -> bool {
        use crate::z80::dataflow::Fact;
        use dez80::instruction::InstructionType;
        use dez80::instruction::Operand;

        fn cprf(r: Option<Operand>, f: Fact) -> bool {
            use dez80::register::RegisterPairType;
            use dez80::register::SingleRegisterType;

            fn cpr(r: SingleRegisterType, f: Fact) -> bool {
                match r {
                    SingleRegisterType::A => f == Fact::A,
                    SingleRegisterType::A_ => f == Fact::A,
                    SingleRegisterType::B => f == Fact::B,
                    SingleRegisterType::B_ => f == Fact::B,
                    SingleRegisterType::C => f == Fact::C,
                    SingleRegisterType::C_ => f == Fact::C,
                    SingleRegisterType::D => f == Fact::D,
                    SingleRegisterType::D_ => f == Fact::D,
                    SingleRegisterType::E => f == Fact::E,
                    SingleRegisterType::E_ => f == Fact::E,
                    SingleRegisterType::H => f == Fact::H,
                    SingleRegisterType::H_ => f == Fact::H,
                    SingleRegisterType::L => f == Fact::L,
                    SingleRegisterType::L_ => f == Fact::L,
                    SingleRegisterType::F => f.is_flag(),
                    SingleRegisterType::F_ => f.is_flag(),
                    SingleRegisterType::IXH => f == Fact::Ixh,
                    SingleRegisterType::IXL => f == Fact::Ixl,
                    SingleRegisterType::IYH => f == Fact::Iyh,
                    SingleRegisterType::IYL => f == Fact::Iyl,
                    SingleRegisterType::PCH => false,
                    SingleRegisterType::PCL => false,
                    SingleRegisterType::SPH => false,
                    SingleRegisterType::SPL => false,
                    SingleRegisterType::I => false,
                    SingleRegisterType::R => false,
                    SingleRegisterType::W => false,
                    SingleRegisterType::Z => false,
                }
            }

            fn cpr2(r: RegisterPairType, f: Fact) -> bool {
                match r {
                    RegisterPairType::AF => f == Fact::A || f.is_flag(),
                    RegisterPairType::AF_ => f == Fact::A || f.is_flag(),
                    RegisterPairType::BC => f == Fact::B || f == Fact::C,
                    RegisterPairType::BC_ => f == Fact::B || f == Fact::C,
                    RegisterPairType::DE => f == Fact::D || f == Fact::E,
                    RegisterPairType::DE_ => f == Fact::D || f == Fact::E,
                    RegisterPairType::HL => f == Fact::H || f == Fact::L,
                    RegisterPairType::HL_ => f == Fact::H || f == Fact::L,
                    RegisterPairType::IX => f == Fact::Ixh || f == Fact::Ixl,
                    RegisterPairType::IY => f == Fact::Iyh || f == Fact::Iyl,
                    RegisterPairType::PC => false,
                    RegisterPairType::SP => false,
                    RegisterPairType::IR => false,
                    RegisterPairType::WZ => false,
                }
            }

            match r {
                Some(Operand::OctetImmediate(_)) => false,
                Some(Operand::DoubletImmediate(_)) => false,
                Some(Operand::OctetImplied(_)) => false,
                Some(Operand::RegisterImplied(r)) => cpr(r, f),
                Some(Operand::RegisterPairImplied(rp)) => cpr2(rp, f),
                Some(Operand::RegisterImpliedBit(r, _)) => cpr(r, f),
                Some(Operand::MemoryDirect(_)) => false,
                Some(Operand::MemoryIndirect(_)) => false,
                Some(Operand::MemoryIndexed(_, _)) => false,
                Some(Operand::MemoryIndexedAndRegister(_, _, _)) => false,
                Some(Operand::MemoryIndirectBit(_, _)) => false,
                Some(Operand::MemoryIndexedBit(_, _, _)) => false,
                Some(Operand::MemoryIndexedBitAndRegister(_, _, _, _)) => false,
                Some(Operand::ProgramCounterRelative(_)) => false,
                Some(Operand::PortDirect(_)) => false,
                Some(Operand::PortIndirect(_)) => false,
                None => false,
            }
        }
        let d = self.decode();

        match (d.r#type, d.destination, fact) {
            (InstructionType::Ret(_), _, _) => false,
            (InstructionType::Jp(_), _, _) => false,
            (InstructionType::Jr(_), _, _) => false,
            (InstructionType::Call(_), _, _) => false,
            (InstructionType::Halt, _, _) => false,
            (InstructionType::Nop, _, _) => false,
            (InstructionType::Push, _, _) => false,
            (InstructionType::Rst(_), _, _) => false,
            (InstructionType::Di, _, _) => false,
            (InstructionType::Ei, _, _) => false,
            (InstructionType::Reti, _, _) => false,
            (InstructionType::Retn, _, _) => false,
            (InstructionType::Djnz, _, f) => f == Fact::B,
            (InstructionType::Rlc, r, f) => cprf(r, f),
            (InstructionType::Rrc, r, f) => cprf(r, f),
            (InstructionType::Rl, r, f) => cprf(r, f),
            (InstructionType::Rr, r, f) => cprf(r, f),
            (InstructionType::Sla, r, f) => cprf(r, f),
            (InstructionType::Sra, r, f) => cprf(r, f),
            (InstructionType::Srl, r, f) => cprf(r, f),
            (InstructionType::Sll, r, f) => cprf(r, f),
            (InstructionType::Ld, r, f) => cprf(r, f),
            (InstructionType::Inc, r, f) => cprf(r, f),
            (InstructionType::Dec, r, f) => cprf(r, f),
            (InstructionType::Cpl, _, f) => f.anh(),
            (InstructionType::Scf, _, f) => f == Fact::Carry,
            (InstructionType::Ccf, _, f) => f.cnh(),
            (InstructionType::Daa, _, f) => f.acph(),
            (InstructionType::Rla, _, f) => f.acnh(),
            (InstructionType::Rra, _, f) => f.acnh(),
            (InstructionType::Rlca, _, f) => f.acnh(),
            (InstructionType::Rrca, _, f) => f.acnh(),
            (InstructionType::Ex, r, f) => cprf(r, f),
            (InstructionType::Exx, _, _) => false,
            (InstructionType::Pop, r, f) => cprf(r, f),
            (InstructionType::Bit, _, _) => true, // probably loads of false positives here
            (InstructionType::Set, r, f) => cprf(r, f),
            (InstructionType::Res, r, f) => cprf(r, f),
            (InstructionType::Add, _, f) => f.achzs(),
            (InstructionType::Adc, _, f) => f.achzs(),
            (InstructionType::Sub, _, f) => f.achzs(),
            (InstructionType::Sbc, _, f) => f.achzs(),
            (InstructionType::And, _, f) => f.acnh(),
            (InstructionType::Xor, _, f) => f.acnh(),
            (InstructionType::Or, _, f) => f.acnh(),
            (InstructionType::Cp, _, f) => f.is_flag(),
            (InstructionType::Out, _, _) => false,
            (InstructionType::In, r, f) => cprf(r, f),
            (InstructionType::Neg, _, f) => f.is_flag() || f == Fact::A,
            (InstructionType::Rrd, _, f) => f.is_flag() || f == Fact::A,
            (InstructionType::Rld, _, f) => f.is_flag() || f == Fact::A,
            (InstructionType::Cpd, _, _) => true,
            (InstructionType::Cpdr, _, _) => true,
            (InstructionType::Cpi, _, _) => true,
            (InstructionType::Cpir, _, _) => true,
            (InstructionType::Im(_), _, _) => true,
            (InstructionType::Outi, _, _) => true,
            (InstructionType::Outd, _, _) => true,
            (InstructionType::Otir, _, _) => true,
            (InstructionType::Otdr, _, _) => true,
            (InstructionType::Ind, _, _) => true,
            (InstructionType::Indr, _, _) => true,
            (InstructionType::Ldi, _, _) => true,
            (InstructionType::Ldir, _, _) => true,
            (InstructionType::Ini, _, _) => true,
            (InstructionType::Inir, _, _) => true,
            (InstructionType::Inva, _, _) => false,
            (InstructionType::Ldd, _, _) => true,
            (InstructionType::Lddr, _, _) => true,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn all_opcodes() {
        use super::Insn;
        use crate::Encode;
        use crate::Iterable;

        let mut insn = Insn::first();
        assert_eq!(insn.len(), 1);
        while insn.step() {
            println!("{}; {:?}", insn, insn);
            let d = insn.decode();

            if !d.ignored_prefixes.is_empty() {
                let prev = insn;
                while !insn.decode().ignored_prefixes.is_empty() {
                    assert!(!insn.step());
                }
                panic!(
                    "{:?} ({}) has ignored prefixes, next one that doesn't is {:?} ({})",
                    prev, prev, insn, insn
                );
            }
        }
    }

    #[test]
    fn next_after_or_ffh() {
        use super::Insn;
        use crate::Encode;
        use crate::Iterable;

        let mut insn = Insn([0xf6, 0xff, 0, 0, 0]);
        println!("{insn} {:?}", insn);
        insn.step();
        println!("{insn} {:?}", insn);
        insn.step();
        println!("{insn} {:?}", insn);
    }

    #[test]
    fn next_after_add_iy_bc() {
        use super::Insn;
        use crate::Encode;
        use crate::Iterable;

        let mut insn = Insn([0xfd, 0x09, 0, 0, 0]);
        println!("{insn} {:?}", insn);
        insn.step();
        println!("{insn} {:?}", insn);
        insn.step();
        println!("{insn} {:?}", insn);
    }
}
