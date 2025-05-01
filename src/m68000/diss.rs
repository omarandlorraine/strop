impl std::fmt::Display for crate::m68000::Insn {
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

impl std::fmt::Debug for crate::m68000::Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{:20} ; {:04x} {:04x} {:04x} {:?}",
            self,
            self.0[0],
            self.0[1],
            self.0[2],
            self.decode().0
        )
    }
}
