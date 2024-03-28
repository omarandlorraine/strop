use crate::Scalar;

impl Scalar for u16 {
    fn random() -> Self {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        if random() {
            rng.gen()
        } else {
            rng.gen_range(0..=100)
        }
    }

    fn as_i32(self) -> i32 {
        self.into()
    }

    fn hamming<T: num::cast::AsPrimitive<u32>>(self, other: T) -> u32 {
        ((self as u32) ^ (other.as_())).count_ones()
    }
}

impl Scalar for i32 {
    fn random() -> Self {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        if random() {
            rng.gen()
        } else {
            rng.gen_range(-100..=100)
        }
    }

    fn as_i32(self) -> i32 {
        self
    }

    fn hamming<T: num::cast::AsPrimitive<u32>>(self, other: T) -> u32 {
        ((self as u32) ^ (other.as_())).count_ones()
    }
}

impl Scalar for u32 {
    fn random() -> Self {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        if random() {
            rng.gen()
        } else {
            rng.gen_range(0..=100)
        }
    }

    fn as_i32(self) -> i32 {
        self as i32
    }

    fn hamming<T: num::cast::AsPrimitive<u32>>(self, other: T) -> u32 {
        (self ^ (other.as_())).count_ones()
    }
}
