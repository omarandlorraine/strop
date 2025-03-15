//! A module defining Subroutine<T>

pub trait MakeReturn {
    fn make_return(&mut self) -> crate::IterationResult;
}

impl<I: crate::Step + MakeReturn> MakeReturn for crate::Sequence<I> {
    fn make_return(&mut self) -> crate::IterationResult {
        let offset_to_last = self.len() - 1;
        if self[offset_to_last].make_return().is_err() {
            self[offset_to_last] = I::first();
            self.push(I::first());
            let offset_to_last = self.len() - 1;
            self[offset_to_last].make_return().unwrap();
        }
        Ok(())
    }
}

/// A type representing a subroutine. This includes the static analysis to make sure that the
/// instruction sequence ends in the appropriate return instruction, etc.
#[derive(Debug, Clone)]
pub struct Subroutine<S>(S);

impl<S> Subroutine<S> {
    /// Wraps the object in the Subroutine struct
    pub fn new(s: S) -> Self {
        Self(s)
    }
}

pub trait ToSubroutine<T: MakeReturn> {
    fn to_subroutine(self) -> Subroutine<Self>
    where
        Self: Sized,
    {
        Subroutine::<Self>::new(self)
    }
}

impl<I, T: crate::Goto<I>> crate::Goto<I> for Subroutine<T> {
    fn goto(&mut self, code: &[I]) {
        self.0.goto(code);
    }
}

impl<T: crate::Disassemble> crate::Disassemble for Subroutine<T> {
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl<S: crate::Step + MakeReturn> crate::Step for Subroutine<S> {
    fn first() -> Self {
        Self(S::first())
    }
    fn next(&mut self) -> crate::IterationResult {
        self.0.next()?;
        self.0.make_return()?;
        Ok(())
    }
}

impl<S: crate::Encode<E>, E> crate::Encode<E> for Subroutine<S> {
    fn encode(&self) -> Vec<E> {
        self.0.encode()
    }
}

impl<T> AsMut<T> for Subroutine<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> AsRef<T> for Subroutine<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<D, T: crate::dataflow::DataFlow<D>> crate::dataflow::DataFlow<D> for Subroutine<T> {
    fn reads(&self, t: &D) -> bool {
        self.0.reads(t)
    }
    fn writes(&self, t: &D) -> bool {
        self.0.writes(t)
    }
    fn modify(&mut self) -> crate::IterationResult {
        self.0.modify()
    }
}
