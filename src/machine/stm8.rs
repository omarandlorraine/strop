use rand::random;
use std::collections::HashMap;

pub struct IndexRegister {
    high: Option<u8>,
    low: Option<u8>,
}

pub struct State {
    a: Option<u8>,
    x: IndexRegister,
    y: IndexRegister,
    heap: HashMap<u16, Option<u8>>,
    carry: Option<bool>,
}

impl State {
    fn get_x(self) -> Option<u16> {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }

    fn get_y(self) -> Option<u16> {
        self.x
            .high
            .zip(self.x.low)
            .map(|(h, l)| u16::from_be_bytes([h, l]))
    }

    fn mem_fetch(self, addr: Option<u16>) -> Option<u8> {
        *addr
            .map(|addr| self.heap.get(&addr).unwrap_or(&None))
            .unwrap_or(&None)
    }
}

enum BitSelect {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

impl From<usize> for BitSelect {
    fn from(item: usize) -> BitSelect {
        match item {
            0 => BitSelect::B0,
            1 => BitSelect::B1,
            2 => BitSelect::B2,
            3 => BitSelect::B3,
            4 => BitSelect::B4,
            5 => BitSelect::B5,
            6 => BitSelect::B6,
            7 => BitSelect::B7,
        }
    }
}

impl From<BitSelect> for usize {
    fn from(item: BitSelect) -> usize {
        match item {
            BitSelect::B0 => 0,
            BitSelect::B1 => 1,
            BitSelect::B2 => 2,
            BitSelect::B3 => 3,
            BitSelect::B4 => 4,
            BitSelect::B5 => 5,
            BitSelect::B6 => 6,
            BitSelect::B7 => 7,
        }
    }
}

// machine specific instruction operand
pub enum Operand {
    A,
    X,
    Y,
    Xh,
    Xl,
    Yh,
    Yl,
    Absolute(u16),
    Immediate8(u8),
    Immediate16(u16),
    IndX,
    IndY,
    // todo: more of these.
}

impl Operand {
    fn get8(self, s: &State) -> Option<u8> {
        match self {
            Operand::A => s.a,
            Operand::X => panic!(),
            Operand::Y => panic!(),
            Operand::Xh => s.x.high,
            Operand::Xl => s.x.low,
            Operand::Yh => s.y.high,
            Operand::Yl => s.y.low,
            Operand::Absolute(addr) => s.mem_fetch(Some(addr)),
            Operand::Immediate8(x) => Some(x),
            Operand::IndX => s.mem_fetch(s.get_x()),
            Operand::IndY => s.mem_fetch(s.get_y()),
        }
    }
}

impl rand::prelude::Distribution<Operand> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Operand {
        use self::Operand::*;
        rng.choose(&[
            A,
            X,
            Y,
            Xh,
            Xl,
            Yh,
            Yl,
            Absolute(random()),
            Immediate8(random()),
            IndX,
            IndY,
        ])
        .unwrap()
        .clone()
    }
}

pub enum Operands {
    None,
    One(Operand),
    Two(Operand, Operand),
    Bits(Operand, BitSelect),
}

impl Operands {
    fn one(self) -> Option<Operand> {
        match self {
            Self::One(o) => Some(o),
            _ => None,
        }
    }

    fn two(self) -> Option<(Operand, Operand)> {
        match self {
            Self::Two(a, b) => Some((a, b)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::Xl => write!(f, "xl"),
            Self::Yh => write!(f, "yh"),
            Self::Xl => write!(f, "xl"),
            Self::Yh => write!(f, "yh"),
            Self::Absolute(addr) => {
                if addr < &256 {
                    write!(f, "${:04x}", addr)
                } else {
                    write!(f, "${:06x}", addr)
                }
            }
            Self::Immediate8(byte) => write!(f, "#${:04x}", byte),
            Self::Immediate16(word) => write!(f, "#${:06x}", word),
            Self::IndX => write!(f, "(x)"),
            Self::IndY => write!(f, "(y)"),
        }
    }
}

pub type Operation = crate::instruction::Operation<State, Operand, (), ()>;
pub type Instruction = crate::instruction::Instruction<'static, State, Operand, (), ()>;

fn rotate_left_through_accumulator(s: &mut State, ind: Operand) {
    let ind = match ind {
        Operand::X => s.x,
        Operand::Y => s.y,
        _ => unimplemented!(),
    };

    let tmp = s.a;
    s.a = ind.high;
    ind.high = ind.low;
    ind.low = tmp;
}

fn rotate_right_through_accumulator(s: &mut State, ind: Operand) {
    let ind = match ind {
        Operand::X => s.x,
        Operand::Y => s.y,
        _ => unimplemented!(),
    };

    let tmp = s.a;
    s.a = ind.low;
    ind.low = ind.high;
    ind.high = tmp;
}

fn flip_x_and_y(d: Operand, sz: usize) -> (Operand, usize) {
    // Many STM8 instructions operate on either X or Y register. For some
    // instructions, this is controlled by an instruction prefix, which of
    // course needs to be accounted for in the instruction's length.
    // So this function changes X for Y and vice versa, and computes the new
    // length. This new length may of course be either discarded or written to
    // the Instruction's length field.
    match d {
        Operand::X => (Operand::Y, sz + 1),
        Operand::Y => (Operand::X, sz - 1),
        Operand::Xl => (Operand::Yl, sz + 1),
        Operand::Yl => (Operand::Xl, sz - 1),
        Operand::Xh => (Operand::Yh, sz + 1),
        Operand::Yh => (Operand::Xh, sz - 1),
        x => (x, sz),
    }
}
