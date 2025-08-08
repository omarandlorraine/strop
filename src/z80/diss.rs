use crate::disassemble::Disassemble;

impl Disassemble for crate::z80::isa::Insn {
    fn dasm(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.decode())
    }

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Encode;

        let bytes = self
            .encode()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<String>>()
            .join(" ");
        write!(f, "{bytes}")
    }
}
