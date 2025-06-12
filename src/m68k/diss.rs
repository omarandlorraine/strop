use crate::m68k::Insn;

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut memory = self.0.as_slice();
        let mut memory = m68000::memory_access::MemoryIter {
            memory: &mut memory,
            next_addr: 0,
        };
        match m68000::instruction::Instruction::from_memory(&mut memory) {
            Ok(ins) => write!(f, "{ins}"),
            Err(e) => panic!("{e:?}"),
        }
    }
}

impl std::fmt::Debug for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Encode;
        let dasm = format!("{self}");
        let len = <Insn as Encode<u16>>::len(self);

        let words = &self.0[..len];

        let hex = words
            .iter()
            .map(|b| format!("{:04x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{:<25} ; {}", dasm, hex)
    }
}
