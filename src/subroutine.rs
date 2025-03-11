//! A module defining Subroutine<T>

pub trait SubroutineT {
    fn make_return(&mut self) -> crate::IterationResult;
}

impl<I: SubroutineT> SubroutineT for crate::Sequence<I> {
    fn make_return(&mut self) -> crate::IterationResult {
        let offset_to_last = self.len() - 1;
        self[offset_to_last].make_return()
    }
}

/// A type representing a subroutine. This includes the static analysis to make sure that the
/// instruction sequence ends in the appropriate return instruction, etc.
#[derive(Debug)]
pub struct Subroutine<S>(S);

impl<S> Subroutine<S> {
    /// Wraps the object in the Subroutine struct
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

pub trait AsSubroutine<T: SubroutineT> {
    fn as_subroutine(self) -> Subroutine<Self> where Self: Sized {
        Subroutine::<Self>::new(self)
    }
}
