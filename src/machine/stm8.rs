use crate::machine::random_absolute;
use crate::machine::random_immediate;
use crate::machine::FlowControl;
use crate::machine::R;
use crate::Datum;
use crate::Machine;
use crate::State;

use crate::machine::rand::prelude::SliceRandom;
use crate::machine::reg_by_name;
use crate::machine::Instruction;
use rand::random;

struct Opcode {
    handler: fn(&Stm8Instruction, &mut State) -> FlowControl,
    name: &'static str,
}

#[derive(Clone, Copy)]
pub struct Stm8Instruction {
    randomizer: fn(&mut Self),
    disassemble: fn(&Self, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    implementation: fn(&Self, &mut State) -> FlowControl,
}

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl Instruction for Stm8Instruction {
    fn randomize(&mut self) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, _s: &mut State) -> FlowControl {
        todo!()
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

const A: Datum = Datum::Register(R::A);
const XL: Datum = Datum::Register(R::Xl);
const YL: Datum = Datum::Register(R::Yl);
const XH: Datum = Datum::Register(R::Xh);
const YH: Datum = Datum::Register(R::Yh);
const X: Datum = Datum::RegisterPair(R::Xh, R::Xl);
const Y: Datum = Datum::RegisterPair(R::Yh, R::Yl);

fn random_imm16() -> Datum {
    let regs = vec![150];
    Datum::Imm16(*regs.choose(&mut rand::thread_rng()).unwrap())
}

fn random_stm8_operand() -> Datum {
    if random() {
        random_immediate()
    } else {
        random_absolute()
    }
}

fn random_register() -> Datum {
    let regs = vec![A, X, Y];
    *regs.choose(&mut rand::thread_rng()).unwrap()
}

fn stm8_reg_by_name(name: &str) -> Result<Datum, &'static str> {
    match name {
        "a" => Ok(A),
        "x" => Ok(X),
        "y" => Ok(Y),
        "xl" => Ok(XL),
        "yl" => Ok(YL),
        "xh" => Ok(XH),
        "yh" => Ok(YH),
        _ => reg_by_name(name),
    }
}

pub const STM8: Machine = Machine {
    name: "stm8",
    reg_by_name: stm8_reg_by_name,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x_width() {
        let mut s = State::new();

        s.set_i16(X, Some(10));
        assert_eq!(10, s.get_i16(XL).unwrap());
        assert_eq!(0, s.get_i16(XH).unwrap());
        assert_eq!(10, s.get_u16(XL).unwrap());
        assert_eq!(0, s.get_u16(XH).unwrap());

        s.set_u16(X, Some(10));
        assert_eq!(10, s.get_u16(XL).unwrap());
        assert_eq!(0, s.get_u16(XH).unwrap());

        s.set_i16(X, Some(-1));
        assert_eq!(-1, s.get_i16(XL).unwrap());
        assert_eq!(-1, s.get_i16(XH).unwrap());

        s.set_i16(X, Some(150));
        assert_eq!(150, s.get_i16(X).unwrap());

        s.set_u16(X, Some(15000));
        assert_eq!(15000, s.get_u16(X).unwrap());

        s.set_i16(X, Some(0));
        assert_eq!(0, s.get_i16(XL).unwrap());
        assert_eq!(0, s.get_i16(XH).unwrap());
    }

    #[test]
    fn reg_names() {
        assert_eq!(stm8_reg_by_name("a").unwrap(), A);
        assert_eq!(stm8_reg_by_name("xl").unwrap(), XL);
        assert_eq!(stm8_reg_by_name("yl").unwrap(), YL);
        assert_eq!(stm8_reg_by_name("xh").unwrap(), XH);
        assert_eq!(stm8_reg_by_name("yh").unwrap(), YH);
        assert_eq!(stm8_reg_by_name("x").unwrap(), X);
        assert_eq!(stm8_reg_by_name("y").unwrap(), Y);
        assert_eq!(stm8_reg_by_name("m6").unwrap(), Datum::Absolute(6));
        assert!(stm8_reg_by_name("n").is_err());
        assert!(stm8_reg_by_name("m").is_err());
    }
}
