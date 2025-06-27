impl std::fmt::Display for crate::armv4t::Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let i = unarm::arm::Ins::new(self.0, &Default::default()).parse(&Default::default());
        write!(f, "{}", i.display(Default::default()))
    }
}

impl std::fmt::Debug for crate::armv4t::Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use crate::Encode;
        let inner: u32 = self.encode()[0];
        let dasm = format!("{self}");
        write!(f, "{dasm:<82} ; {inner:#010x}")
    }
}
