use smallvec::SmallVec;
use std::borrow::Cow;

pub mod stm8;

#[derive(Clone, Copy)]
pub struct Machine {
    pub name: &'static str,
}

impl<State, Operand, OUD, IUD> std::fmt::Display for Instruction<'_, State, Operand, OUD, IUD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.disassemble)(f, self)
    }
}

impl<State, Operand, OUD, IUD> Instruction<'_, State, Operand, OUD, IUD> {
    pub fn randomize(&mut self) {
        (self.randomizer)(self);
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn operate(&self, s: &mut State) {
        (self.implementation)(self, s)
    }
}

#[derive(Clone)]
pub struct Operation<'a, State, Operand, OUD, IUD> {
    pub disassemble: fn(&Instruction<'a, State, Operand, OUD, IUD>) -> Cow<'static, str>,
    pub mutate: fn(&mut Instruction<'a, State, Operand, OUD, IUD>) -> (),
    pub execute: fn(&Instruction<'a, State, Operand, OUD, IUD>, &mut State) -> u64,
    pub userdata: OUD,
}

#[derive(Clone)]
pub struct Instruction<'op, State, Operand, OUD, IUD> {
    pub operation: &'op Operation<'op, State, Operand, OUD, IUD>,
    pub operands: SmallVec<[Operand; 4]>,
    pub userdata: IUD,
}
