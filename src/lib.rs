mod backends;

trait Instruction {
    fn new() -> Self;
    fn mutate(&mut self);
    fn length(&self) -> usize;
    fn disassemble(&self) -> String;
}
