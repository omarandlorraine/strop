use crate::Mutate;
use crate::armv4t::Insn;

impl Mutate for Insn {
    fn random() -> Self {
        // rejection sampling. In the relatively unlikely event that the randomly generated
        // instruction is wrong somehow, then just discard it! and try again.
        loop {
            let mut s = Self(rand::random());
            if s.fixup().is_ok() {
                return s;
            }
        }
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

        while self.fixup().is_err() {
            self.0 = rand::random()
        }
    }
}
