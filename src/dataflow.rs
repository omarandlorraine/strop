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
pub struct NotLiveIn<Datum>(Datum);

/// A constraint for asserting that a datum (that is, the type parameter representing, say, a
/// register, status flag, memory location, etc.) is not live at the end of a sequence.
///
/// The `report` method will point out dead code, where a value is written to the locus and there
/// is no subsequent read from the locus.
#[derive(Debug)]
pub struct NotLiveOut<Datum>(Datum);

impl<Datum> NotLiveOut<Datum> {
    /// builds a new `NotLiveOut` struct.
    pub fn new(datum: Datum) -> NotLiveOut<Datum> {
        NotLiveOut::<Datum>(datum)
    }
}

impl<Insn, Datum> Constrain<Insn> for NotLiveIn<Datum>
where
    Insn: DataFlow<Datum> + Clone + Iterable,
    Datum: std::fmt::Debug,
{
    fn fixup(&self, seq: &mut Sequence<Insn>) -> Option<(usize, &'static str)> {
        if let Some(offset) = self.check(seq) {
            if self.check_reads(seq) == Some(offset) {
                seq.mut_at(Insn::modify, offset);
                return Some((offset, "reads uninitialized value"));
            } else if self.check_writes(seq) == Some(offset) {
                seq.mut_at(Insn::modify, offset);
                return Some((offset, "writes a value that's never read"));
            } else {
                unreachable!();
            }
        }
        None
    }
}

impl<Datum> NotLiveIn<Datum> {
    /// builds a new `NotLiveIn` struct.
    pub fn new(datum: Datum) -> NotLiveIn<Datum> {
        NotLiveIn::<Datum>(datum)
    }

    fn check_reads<Insn: DataFlow<Datum>>(&self, seq: &Sequence<Insn>) -> Option<usize> {
        for (offset, i) in seq.iter().enumerate() {
            if i.writes(&self.0) {
                // The sequence writes to the datum before any reads, which is okay.
                return None;
            }
            if i.reads(&self.0) {
                // The sequence reads from the datum before any writes, which is bad.
                return Some(offset);
            }
        }
        None
    }

    fn check_writes<Insn: DataFlow<Datum>>(&self, seq: &Sequence<Insn>) -> Option<usize> {
        let mut found_write: Option<usize> = None;

        for (offset, i) in seq.iter().enumerate() {
            if i.writes(&self.0) {
                // The sequence writes to the datum before any reads, which is okay so long as the
                // value is read by a later instruction
                found_write = Some(offset);
            }
            if i.reads(&self.0) && found_write.is_some() {
                // The sequence reads from the datum before any writes, which is bad.
                return None;
            }
        }
        found_write
    }

    fn check<Insn: DataFlow<Datum>>(&self, seq: &Sequence<Insn>) -> Option<usize> {
        match (self.check_reads(seq), self.check_writes(seq)) {
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
