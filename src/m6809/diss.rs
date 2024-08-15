use crate::m6809::isa::Insn;

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fn rs(
            f: &mut std::fmt::Formatter<'_>,
            i9n: &'static str,
            rs: u8,
            other_stack: &'static str,
        ) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "{} ", i9n)?;
            if rs & 0x01 != 0 {
                write!(f, "cc ")?;
            }
            if rs & 0x02 != 0 {
                write!(f, "a ")?;
            }
            if rs & 0x04 != 0 {
                write!(f, "b ")?;
            }
            if rs & 0x08 != 0 {
                write!(f, "dp ")?;
            }
            if rs & 0x10 != 0 {
                write!(f, "x ")?;
            }
            if rs & 0x20 != 0 {
                write!(f, "y ")?;
            }
            if rs & 0x40 != 0 {
                write!(f, "{} ", other_stack)?;
            }
            if rs & 0x80 != 0 {
                write!(f, "pc ")?;
            }
            Ok(())
        }
        use crate::Encode;
        use emu6809::mem::MemBlock;

        match self.encode()[0] {
            0x34 => rs(f, "pshs", self.encode()[1], "u"),
            0x35 => rs(f, "puls", self.encode()[1], "u"),
            0x36 => rs(f, "pshu", self.encode()[1], "s"),
            0x37 => rs(f, "pulu", self.encode()[1], "s"),
            _ => {
                let mut memblock: MemBlock<emu6809::byteorder::LittleEndian> =
                    MemBlock::from_data(0, "test_i9n", &self.encode(), true);
                write!(
                    f,
                    "{}",
                    emu6809::diss::Diss::new().diss(&mut memblock, 0).text
                )
            }
        }
    }
}

impl std::fmt::Debug for Insn {
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
