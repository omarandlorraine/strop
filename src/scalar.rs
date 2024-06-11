use crate::Scalar;

fn average<T>(a: T, b: T) -> f32
where
    T: Into<f32> + Copy,
{
    let a: f32 = a.into();
    let b: f32 = b.into();
    let sum = a + b;
    let average = sum / 2.0;
    average
}

fn difference<T>(a: T, b: T) -> f32
where
    T: Into<f32> + Copy,
{
    let a: f32 = a.into();
    let b: f32 = b.into();
    (a - b).abs()
}

fn hamming<T>(a: T, b: T) -> f32
where
    T: Into<u32> + Copy,
{
    let a: u32 = a.into();
    let b: u32 = b.into();
    (a ^ b).count_ones() as f32
}

fn compare<T>(a: T, b: T) -> f32
where
    T: Into<u32> + Copy + num::cast::AsPrimitive<f32>
{
    average(hamming(a, b), difference(a.as_(), b.as_()))
}

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

    fn cmp<T: num::cast::AsPrimitive<u32>>(self, other: T) -> f32 {
        compare(self as u32, other.as_())
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

    fn cmp<T: num::cast::AsPrimitive<u32>>(self, other: T) -> f32 {
        compare(self as u32, other.as_())
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

    fn cmp<T: num::cast::AsPrimitive<u32>>(self, other: T) -> f32 {
        compare(self as u32, other.as_())
    }
}
