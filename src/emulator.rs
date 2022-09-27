pub trait Emulator {
    fn load(&mut self, org: usize, bytes: &mut dyn Iterator<Item = u8>);
    fn run(&mut self, org: usize, budget: u32, bytes: &mut dyn Iterator<Item = u8>);
}
