use crate::HammingDistance;

impl HammingDistance<u8> for u8 {
    fn hamming_distance(&self, other: u8) -> f32 {
        (self ^ other).pop_count() as f32
    }
}

impl HammingDistance<i8> for u8 {
    fn hamming_distance(&self, other: i8) -> f32 {
        ((self as i8) ^ other).pop_count() as f32
    }
}

impl HammingDistance<u8> for i8 {
    fn hamming_distance(&self, other: u8) -> f32 {
        other.hamming_distance(self)
    }
}

impl HammingDistance<i8> for i8 {
    fn hamming_distance(&self, other: i8) -> f32 {
        other.hamming_distance(self)
    }
}

impl HammingDistance<i8> for u32 {
    fn hamming_distance(&self, other: i8) -> f32 {
        self.to_be_bytes()[0].hamming_distance(other)
    }
}
