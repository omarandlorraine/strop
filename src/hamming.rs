use crate::HammingDistance;

impl HammingDistance<u8> for u8 {
    fn hamming_distance(self, other: u8) -> f32 {
        (self ^ other).count_ones() as f32
    }
}

impl HammingDistance<u16> for u16 {
    fn hamming_distance(self, other: u16) -> f32 {
        (self ^ other).count_ones() as f32
    }
}

impl HammingDistance<i8> for u8 {
    fn hamming_distance(self, other: i8) -> f32 {
        ((self as i8) ^ other).count_ones() as f32
    }
}

impl HammingDistance<u8> for i8 {
    fn hamming_distance(self, other: u8) -> f32 {
        other.hamming_distance(self)
    }
}

impl HammingDistance<i8> for i8 {
    fn hamming_distance(self, other: i8) -> f32 {
        (self ^ other).count_ones() as f32
    }
}

impl HammingDistance<i8> for u32 {
    fn hamming_distance(self, other: i8) -> f32 {
        self.to_be_bytes()[0].hamming_distance(other)
    }
}

impl HammingDistance<u32> for u32 {
    fn hamming_distance(self, other: u32) -> f32 {
        (self ^ other).count_ones() as f32
    }
}

impl HammingDistance<u16> for u32 {
    fn hamming_distance(self, other: u16) -> f32 {
        self.to_le_bytes()[0].hamming_distance(other.to_le_bytes()[0])
            + self.to_le_bytes()[1].hamming_distance(other.to_le_bytes()[1])
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn hammings() {
        use super::HammingDistance;

        assert_eq!(1u8.hamming_distance(1u8), 0.0);
        assert_eq!(1u8.hamming_distance(1i8), 0.0);
        assert_eq!(0u8.hamming_distance(1i8), 1.0);
        assert_eq!(1u32.hamming_distance(1u16), 0.0);
        assert_eq!(1u32.hamming_distance(1u32), 0.0);
    }
}
