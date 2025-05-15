//! A module defining Subroutine<T>

use crate::Branch;
use crate::BruteforceSearch;
use crate::Encode;
use crate::Sequence;

/// A type representing a subroutine. This includes the static analysis to make sure that the
/// instruction sequence ends in the appropriate return instruction, etc.
#[derive(Debug, Clone)]
pub struct Subroutine<Insn: Encode<u8> + Branch, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>(
    S,
    std::marker::PhantomData<Insn>,
);

impl<Insn: Encode<u8> + Branch + crate::Step + Clone + ShouldReturn> Default
    for Subroutine<Insn, Sequence<Insn>>
{
    fn default() -> Self {
        use crate::subroutine::ToSubroutine;
        use crate::Step;
        crate::Sequence::<Insn>::first().to_subroutine()
    }
}

pub trait ShouldReturn {
    fn should_return(&self) -> Result<(), crate::StaticAnalysis<Self>>
    where
        Self: Sized;

    fn allowed_in_subroutine(&self) -> Result<(), crate::StaticAnalysis<Self>>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<Insn: Encode<u8> + Branch, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    AsRef<Sequence<Insn>> for Subroutine<Insn, S>
{
    fn as_ref(&self) -> &Sequence<Insn> {
        self.0.as_ref()
    }
}

impl<Insn: Encode<u8> + Branch, S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>>
    Subroutine<Insn, S>
{
    /// Wraps the object in the Subroutine struct
    pub fn new(s: S) -> Self {
        Self(s, std::marker::PhantomData)
    }
}

impl<
        Insn: crate::Encode<u8> + Branch + ShouldReturn,
        S: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>,
    > BruteforceSearch<Insn> for Subroutine<Insn, S>
{
    fn analyze_this(&self) -> Result<(), crate::StaticAnalysis<Insn>>
    where
        Self: Sized,
    {
        let seq = self.0.as_ref();
        let offs = seq.last_instruction_offset();
        if let Err(sa) = seq[offs].should_return() {
            return Err(sa.set_offset(offs));
        }

        let start_addresses = seq
            .iter()
            .map(|insn| insn.len())
            .scan(0, |sum, x| {
                *sum += x;
                Some(*sum as isize)
            })
            .collect::<Vec<isize>>();

        let mut backward = 0;
        for insn in seq.iter() {
            let permissibles = start_addresses
                .iter()
                .flat_map(|x| x.checked_sub(backward))
                .collect::<Vec<isize>>();
            insn.branch_fixup(&permissibles)?;
            backward += insn.len() as isize;

            insn.allowed_in_subroutine()?;
        }
        Ok(())
    }

    fn inner(&mut self) -> &mut dyn BruteforceSearch<Insn> {
        &mut self.0
    }
}

pub trait ToSubroutine<T: Encode<u8> + ShouldReturn + Branch> {
    fn to_subroutine(self) -> Subroutine<T, Self>
    where
        Self: Sized + BruteforceSearch<T>,
        Self: AsRef<Sequence<T>>,
    {
        Subroutine::<T, Self>::new(self)
    }
}

impl<
        Insn: Encode<u8> + Branch,
        S: crate::Disassemble + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>,
    > crate::Disassemble for Subroutine<Insn, S>
{
    fn dasm(&self) {
        self.0.dasm()
    }
}

impl<
        Insn: Encode<u8> + ShouldReturn + Branch,
        S: crate::Step + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>,
    > crate::Step for Subroutine<Insn, S>
{
    fn first() -> Self {
        let mut r = Self(S::first(), std::marker::PhantomData);
        r.step();
        r.fixup();
        r
    }
    fn next(&mut self) -> crate::IterationResult {
        self.0.next()?;
        self.fixup();
        Ok(())
    }
}

impl<
        Insn: Encode<u8> + Branch,
        S: crate::Encode<E> + BruteforceSearch<Insn> + AsRef<Sequence<Insn>>,
        E,
    > crate::Encode<E> for Subroutine<Insn, S>
{
    fn encode(&self) -> Vec<E> {
        self.0.encode()
    }
}

impl<Insn: Encode<u8> + Branch, T: BruteforceSearch<Insn> + AsRef<Sequence<Insn>>> AsMut<T>
    for Subroutine<Insn, T>
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
