//! Module for representing ARMv4T machine code instructions.

/// Represents an ARMv4T machine code instruction.
#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct Insn(pub(crate) u32);

impl crate::Iterable for Insn {
    fn first() -> Self {
        Insn(0)
    }

    fn step(&mut self) -> bool {
        if self.0 > 0xfffffffe {
            false
        } else {
            self.0 += 1;
            self.fixup();
            true
        }
    }
}

impl crate::Encode<u8> for Insn {
    fn encode(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl crate::Encode<u32> for Insn {
    fn encode(&self) -> Vec<u32> {
        vec![self.0]
    }
}

impl Insn {
    /// Decodes the instruction and returns an `unarm::ParsedIns`
    pub fn decode(&self) -> unarm::ParsedIns {
        unarm::v4t::arm::Ins::new(self.0, &Default::default()).parse(&Default::default())
    }

    /// No matter the `Insn`'s value, if it does not encode a valid ARMv4T machine code
    /// instruction, this method mutates it so that it does.
    pub fn fixup(&mut self) {
        fn exclude(i: &mut Insn, rng: std::ops::RangeInclusive<u32>) {
            if rng.contains(&i.0) {
                i.0 = *rng.end() + 1;
                assert!(i.is_valid(), "{:?}", i);
            }
        }

        if self.0 & 0x0e000000 == 0x00000000 {
            let rotate: u8 = ((self.0 >> 4) & 0x000000ff).try_into().unwrap();

            if rotate & 0x01 == 0 {
                // shifting by a five-bit unsigned integer, nothing to check
            } else {
                // shifting by an amount specified in a register; we need to check that bit 7 of
                // the instruction is 0, otherwise it is an undefined instruction.
                if self.0 & 0x00000080 != 0 {
                    self.0 |= 0xff;
                    self.0 += 1;
                }
            }
        }
        exclude(self, 0x01000000..=0x010eff8f);

        /*
        if self.0 & 0x0d900000 == 0x01800000 {
            // the instruction is one of: tst, teq, cmp, cmn, but the the S bit is not set!
            // So for this instruction to be valid we need to set the S bit
            self.0 |= 0x00100000
        }
        */
        if self.0 & 0x0fffffff == 0x012fff1f {
            // It's a bx{cond} pc instruction, which is undefined
            self.0 += 1;
        }
    }

    /// Returns `true` iff the `Insn` represents a valid ARMv4T machine code instruction.
    pub fn is_valid(&self) -> bool {
        if self.0 & 0x0c000000 == 0x0c000000 {
            // coprocessor instructions are not implemented in the emulator
            return false;
        } else if self.0 & 0x0e000010 == 0x06000010 {
            // this range of instructions is undefined
            return false;
        }

        let d = self.decode();

        if d.mnemonic.starts_with("bx") {
            // A Branch and Exchange instruction with PC as its operand is undefined behaviour
            if let unarm::args::Argument::Reg(reg) = d.args[0] {
                return reg.reg != unarm::args::Register::Pc;
            }
        }

        true
    }
}

#[cfg(test)]
mod test {

    fn emulator_knows_it(i9n: super::Insn) -> bool {
        use crate::Encode;
        use armv4t_emu::{reg, Cpu, ExampleMem, Mode};
        let mut mem = ExampleMem::new_with_data(&i9n.encode());
        let mut cpu = Cpu::new();
        cpu.reg_set(Mode::User, reg::PC, 0x00);
        cpu.reg_set(Mode::User, reg::CPSR, 0x10);
        cpu.step(&mut mem)
    }

    #[test]
    #[ignore]
    fn all_instructions() {
        use crate::Iterable;

        let mut i = super::Insn(0xff7affff);
        // let mut i = super::Insn::first();

        while i.step() {
            i.decode();

            // check that the instruction can be disassembled
            format!("{}", i);
            assert_eq!(format!("{:?}", i).len(), 95, "{:?}", i);

            // println!("{:?}", i);

            assert!(!format!("{:?}", i).contains("illegal"), "{:?}", i);

            // check that the increment method does not visit invalid instructions; this will in
            // turn validate the fixup method.
            if !i.is_valid() {
                let beginning = i;
                let mut end = i;
                while !end.is_valid() {
                    end.step();
                }
                panic!("found a range of illegal instructions visited by the .increment method: 0x{:08x}..=0x{:08x}",
                       beginning.0, end.0
                      );
            }

            // check that the emulator can execute the instruction
            if !emulator_knows_it(i) {
                let beginning = i;
                let mut end = i;
                while !emulator_knows_it(i) {
                    end = i;
                    println!("the emulator can't run {:?}", i);
                    i.step();
                }
                println!("the range is {:?}..{:?} inclusive", beginning.0, end.0);
                panic!("found a range of instructions visited by the .increment method that the emulator doesn't know");
            }
        }
    }
}
