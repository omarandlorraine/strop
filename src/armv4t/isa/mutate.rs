use crate::Mutate;
use crate::armv4t::Insn;

impl Mutate for Insn {
    fn random() -> Self {
        let mut s = Self(rand::random());
        s.fixup();
        s
    }

    fn mutate(&mut self) {
        use rand::Rng;

        if rand::random() {
            // could flip a bit in the instruction word
            let mask: u32 = 1 << rand::rng().random_range(0..32);
            self.0 ^= mask;
        } else {
            // could completely change the instruction word to something completely different
            self.0 = rand::random()
        }

        while !self.fixup() {
            self.0 = rand::random()
        }
    }
}
