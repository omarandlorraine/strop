use crate::machine::rand::prelude::SliceRandom;
use std::collections::HashMap;
extern crate num;
extern crate rand;
use num::traits::{WrappingAdd, WrappingSub};
use std::convert::TryInto;

pub mod mos6502;
pub mod stm8;
pub mod x80;

pub trait Instruction: std::fmt::Display + Clone + Sized {
    type State;
    fn randomize(&mut self);
    fn len(&self) -> usize;
    fn operate(&self, s: &mut Self::State);
    fn random() -> Self
    where
        Self: Sized;
}
