//! Disassembly.

pub trait Disassemble {
    //! A trait for printing out the disassembly of an instruction. Expect all types representing
    //! machine instructions to impl this trait.

    /// Disassemble
    fn dasm(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>;

    /// Disassemble, including a binary representation
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>;
}

/// A struct for writing out a sequence of instructions.
///
/// This comes with useful methods for printing to stdout and whatever, but if you want to do
/// anything else like writing to a file or a string, then this struct also impls `std::fmt::Display` so you can do what you want.
#[derive(Debug)]
pub struct Disassembly<'a, T: Disassemble>(&'a [T], &'a str);

impl<'a, T: Disassemble> std::fmt::Display for Disassembly<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.1)?;
        for i in self.0 {
            write!(f, "    ")?;
            i.dasm(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
impl<'a, T: Disassemble> Disassembly<'a, T> {
    /// Construct a new disassembly
    pub fn new(t: &'a [T], name: &'a str) -> Self {
        Self(t, name)
    }

    /// Print the disassembly to stdout
    pub fn print(t: &'a [T], name: &'a str) {
        let d = Self::new(t, name);
        println!("{d}");
    }
}
