//! A strop backend targetting the Zilog Z80.

mod instruction_set;
pub use instruction_set::Instruction;
pub(crate) mod data;
mod emu;
pub use emu::Emulator;
mod sdcccall;

#[cfg(test)]
mod dasm;

#[cfg(test)]
mod tests {
    use super::Instruction;
    use crate::Instruction as _;
    use crate::backends::x80::X80;

    #[test]
    fn std_x80_tests() {
        crate::backends::x80::tests::std_x80_tests::<Instruction>();
    }

    #[test]
    fn unique_disassembly() {
        crate::generic_unit_tests::disassemblies_unique(Instruction::first(), None);
    }

    #[test]
    fn known_dupes() {
        crate::generic_unit_tests::list_all_encodings(
            "ex (sp), hl",
            Instruction::first(),
            Instruction::from_bytes(&[0xff]),
        );
        crate::generic_unit_tests::list_all_encodings(
            "ld (bc), a",
            Instruction::first(),
            Instruction::from_bytes(&[0xff]),
        );
    }

    #[test]
    fn prefixation() {
        assert!(Instruction::from_bytes(&[0xED]).is_none());
        // no such instruction
        assert!(Instruction::from_bytes(&[0xdd, 0x00, 0xff, 0xff, 0xff]).is_none());

        let mut insn = Instruction::from_bytes(&[0xdc, 0xff, 0xff]).unwrap();
        println!("{:?}", insn.to_bytes());
        println!("{:?}", insn.decode());
        insn.increment().unwrap();
        assert_eq!(0xdd, insn.to_bytes()[0]);
    }

    #[test]
    fn dd_prefix() {
        use crate::backends::x80::data::ReadWrite;

        for op in 0..=255 {
            if let Some((i, dd)) = Instruction::from_bytes(&[op, 0, 0, 0, 0])
                .zip(Instruction::from_bytes(&[0xdd, op, 0, 0, 0]))
            {
                let fd = Instruction::from_bytes(&[0xfd, op, 0, 0, 0]).unwrap();
                assert_ne!(format!("{i}"), format!("{dd}"),);
                assert_ne!(format!("{fd}"), format!("{dd}"),);

                assert!(dd.decode().bytes > 1, "{op:x}");
                assert!(fd.decode().bytes > 1, "{op:x}");

                assert_eq!(dd.decode().h, ReadWrite::N, "{dd:?}");
                assert_eq!(dd.decode().l, ReadWrite::N, "{dd:?}");
                assert!(
                    ((dd.decode().ixh != ReadWrite::N) || (dd.decode().ixl != ReadWrite::N)),
                    "{dd:?}, {:?}",
                    dd.decode()
                );
                assert_eq!(dd.decode().iyh, ReadWrite::N, "{dd:?}");
                assert_eq!(dd.decode().iyl, ReadWrite::N, "{dd:?}");

                assert_eq!(fd.decode().h, ReadWrite::N, "{fd:?}");
                assert_eq!(fd.decode().l, ReadWrite::N, "{fd:?}");
                assert_eq!(fd.decode().ixh, ReadWrite::N, "{fd:?}");
                assert_eq!(fd.decode().ixl, ReadWrite::N, "{fd:?}");
                assert!(
                    (fd.decode().iyh != ReadWrite::N) || (fd.decode().iyl != ReadWrite::N),
                    "{fd:?}"
                );
            }
        }
    }
}
