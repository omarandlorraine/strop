use crate::Disassemble;
use std::fmt::Display;

impl Display for crate::z80::isa::Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.decode())
    }
}

impl std::fmt::Debug for crate::z80::isa::Insn {
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

impl Disassemble for crate::z80::isa::Insn {
    fn dasm(&self) {
        println!("\t{}", self);
    }
}
