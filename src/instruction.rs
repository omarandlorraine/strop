use smallvec::SmallVec;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Operation<State, Operand, OUD, IUD> {
    pub disassemble: for<'a> fn(&Instruction<'a, State, Operand, OUD, IUD>) -> Cow<'static, str>,
    pub mutate: for<'a> fn(&mut Instruction<'a, State, Operand, OUD, IUD>) -> (),
    pub execute: for<'a> fn(&Instruction<'a, State, Operand, OUD, IUD>, &mut State) -> u64,
    pub userdata: OUD,
}

#[derive(Clone)]
pub struct Instruction<'op, State, Operand, OUD, IUD> {
    pub operation: &'op Operation<State, Operand, OUD, IUD>,
    pub operands: SmallVec<[Operand; 4]>,
    pub userdata: IUD,
}
