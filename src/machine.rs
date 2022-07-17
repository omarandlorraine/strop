extern crate num;
extern crate rand;

pub mod mos6502;
pub mod stm8;
pub mod x80;

pub trait Instruction: std::fmt::Display + Clone + Sized {
    type State: Default;
    fn randomize(&mut self);
    fn len(&self) -> usize;
    fn operate(&self, s: &mut Self::State);
    fn random() -> Self
    where
        Self: Sized;
}
