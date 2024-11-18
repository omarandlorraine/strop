//! This module contains miscellaneous conveniences for performing dataflow analysis on code
//! sequences.

use crate::Constrain;
use crate::DataFlow;
use crate::Iterable;
use crate::Sequence;

/// A constraint for asserting that a datum (that is, the type parameter representing, say, a
/// register, status flag, memory location, etc.) is not live at the beginning of a sequence.
///
/// The `report` method will point out uses of uninitialized data, and the fixup method will modify
/// the sequence so that it doesn't read from the datum without initializing it.
#[derive(Debug)]
pub struct NotLiveIn<'a, Insn, Datum>
where
    Insn: DataFlow<Datum>,
{
    seq: &'a mut Sequence<Insn>,
    datum: Datum,
}

/// A constraint for asserting that a datum (that is, the type parameter representing, say, a
/// register, status flag, memory location, etc.) is not live at the end of a sequence.
///
/// The `report` method will point out dead code, where a value is written to the locus and there
/// is no subsequent read from the locus.
#[derive(Debug)]
pub struct NotLiveOut<'a, Insn, Datum>
where
    Insn: DataFlow<Datum>,
{
    seq: &'a mut Sequence<Insn>,
    datum: Datum,
}

impl<Insn, Datum> NotLiveOut<'_, Insn, Datum>
where
    Insn: DataFlow<Datum> + Clone,
{
    /// builds a new `NotLiveOut` struct.
    pub fn new<'a>(seq: &'a mut Sequence<Insn>, datum: Datum) -> NotLiveOut<'a, Insn, Datum> {
        NotLiveOut::<'a, Insn, Datum> { seq, datum }
    }

    fn check(&self) -> Option<usize> {
        let mut found_write = None;

        for (offset, i) in self.seq.clone().into_iter().enumerate() {
            if i.reads(&self.datum) {
                // The sequence writes to the datum before any reads, which is okay.
                found_write = None;
            }
            if i.writes(&self.datum) {
                // The sequence reads from the datum before any writes, which is bad.
                found_write = Some(offset);
            }
        }
        found_write
    }
}

impl<Insn, Datum> Constrain<Insn> for NotLiveIn<'_, Insn, Datum>
where
    Insn: DataFlow<Datum> + Clone + Iterable,
    Datum: std::fmt::Debug,
{
    fn fixup(&mut self) {
        while let Some(offset) = self.check() {
            self.seq.mut_at(Insn::modify, offset);
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        if self.check_reads() == Some(offset) {
            vec![format!("Reading {:?} before it's initialized", self.datum)]
        } else if self.check_writes() == Some(offset) {
            vec![format!("Writing to {:?} and it's never read", self.datum)]
        } else {
            vec![]
        }
    }
}

impl<Insn, Datum> NotLiveIn<'_, Insn, Datum>
where
    Insn: DataFlow<Datum> + Clone,
{
    /// builds a new `NotLiveIn` struct.
    pub fn new<'a>(seq: &'a mut Sequence<Insn>, datum: Datum) -> NotLiveIn<'a, Insn, Datum> {
        NotLiveIn::<'a, Insn, Datum> { seq, datum }
    }

    fn check_reads(&self) -> Option<usize> {
        for (offset, i) in self.seq.clone().into_iter().enumerate() {
            if i.writes(&self.datum) {
                // The sequence writes to the datum before any reads, which is okay.
                return None;
            }
            if i.reads(&self.datum) {
                // The sequence reads from the datum before any writes, which is bad.
                return Some(offset);
            }
        }
        None
    }

    fn check_writes(&self) -> Option<usize> {
        let mut found_write: Option<usize> = None;

        for (offset, i) in self.seq.clone().into_iter().enumerate() {
            if i.writes(&self.datum) {
                // The sequence writes to the datum before any reads, which is okay so long as the
                // value is read by a later instruction
                found_write = Some(offset);
            }
            if i.reads(&self.datum) && found_write.is_some() {
                // The sequence reads from the datum before any writes, which is bad.
                return None;
            }
        }
        found_write
    }

    fn check(&self) -> Option<usize> {
        match (self.check_reads(), self.check_writes()) {
            (Some(a), Some(b)) => {
                if a < b {
                    Some(a)
                } else {
                    Some(b)
                }
            }
            (None, Some(a)) => Some(a),
            (Some(a), None) => Some(a),
            (None, None) => None,
        }
    }
}

impl<Insn, Datum> Constrain<Insn> for NotLiveOut<'_, Insn, Datum>
where
    Insn: DataFlow<Datum> + Clone + Iterable,
    Datum: std::fmt::Debug,
{
    fn fixup(&mut self) {
        while let Some(offset) = self.check() {
            self.seq.mut_at(Insn::modify, offset);
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        if self.check() == Some(offset) {
            vec![format!("Write to {:?} is dead", self.datum)]
        } else {
            vec![]
        }
    }
}
