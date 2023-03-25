pub use rand;
mod backends;

#[macro_export]
macro_rules! randomly {
    (@ $n:expr, ($action:block $($rest:block)*), ($($arms:tt,)*)) => {
        randomly!(@ $n + 1, ($($rest)*), (($n, $action), $($arms,)*))
    };
    (@ $n:expr, (), ($(($m:expr, $action:block),)*)) => {{
        use $crate::rand::{thread_rng, Rng};
        let i = thread_rng().gen_range(0, $n);
        match i {
            $(x if x == $m => $action)*
            _ => panic!(),
        }
    }};
    ($($action:block)*) => {
        randomly!(@ 0, ($($action)*), ())
    };
}

trait Emulator {
    type Addr;
    type Insn;

    fn run(&mut self, org: Self::Addr, prog: Vec<Self::Insn>);
}

trait Instruction {
    fn new() -> Self;
    fn mutate(&mut self);
    fn length(&self) -> usize;
    fn disassemble(&self) -> String;
}
