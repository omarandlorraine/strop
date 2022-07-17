use crate::machine::rand::prelude::SliceRandom;
use crate::machine::Instruction;
use rand::random;

pub struct IndexRegister {
    high: Option<u8>,
    low: Option<u8>,
}

pub struct Stm8 {
    a: Option<u8>,
    x: IndexRegister,
    y: IndexRegister,
}

struct Opcode {
    handler: fn(&Stm8Instruction, &mut Stm8),
    name: &'static str,
}

#[derive(Clone, Copy)]
pub struct Stm8Instruction {
    randomizer: fn(&mut Self),
    disassemble: fn(&Self, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
    implementation: fn(&Self, &mut Stm8),
}

impl std::fmt::Display for Stm8Instruction {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl Instruction for Stm8Instruction {
    type State = Stm8;

    fn randomize(&mut self) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn operate(&self, _s: &mut Stm8) {
        todo!()
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

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
