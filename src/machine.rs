extern crate num;
extern crate rand;

pub mod mos6502;
pub mod stm8;
pub mod x80;

pub trait Strop {
    fn mutate(&mut self);
    fn random() -> Self
    where
        Self: Sized;
}

pub trait Instruction: std::fmt::Display + Clone + Sized {
    type State: Default;
    fn randomize(&mut self);
    fn len(&self) -> usize;
    fn operate(&self, s: &mut Self::State);
    fn new() -> Self
    where
        Self: Sized;
}
