//! This module contains miscellaneous conveniences for performing dataflow analysis on code
//! sequences.

use crate::Sequence;
use crate::StaticAnalysis;

/// Implement this trait on an instruction to communicate that the instruction reads from or writes
/// to a datum of some type. For example, if you have a type representing the register file, and
/// some type represents the machine's instruction, these methods can communicate what use the
/// instruction makes of each register.
pub trait DataFlow<Datum> {
    /// Returns `true` iff the instruction reads from `datum`.
    fn reads(&self, datum: &Datum) -> bool;

    /// Returns `true` iff the instruction writes to `datum`.
    fn writes(&self, datum: &Datum) -> bool;

    /// Returns a `StaticAnalysis` for advancing the instruction.
    fn sa(&self) -> StaticAnalysis<Self>
    where
        Self: Sized;
}

/// Returns a static analysis modifying any instructions that read from or writes to `datum`.
pub fn leave_alone<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    sequence
        .iter()
        .enumerate()
        .find(|(_offs, i)| i.reads(datum) || i.writes(datum))
        .map(|(offs, i)| i.sa().set_offset(offs))
        .map_or(Ok(()), Err)
}

/// Returns a static analysis modifying any instructions that read from or writes to `datum`, but
/// ignores the last instruction in the sequence. Handy for cases like leaf subroutines on MIPS or ARM,
/// where only the last instruction should use the link register.
pub fn leave_alone_except_last<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    let len = sequence.len();
    sequence
        .iter()
        .take(len - 1)
        .enumerate()
        .find(|(_offs, i)| i.reads(datum) || i.writes(datum))
        .map(|(offs, i)| i.sa().set_offset(offs))
        .map_or(Ok(()), Err)
}

/// If the sequence reads from `datum` before writing to it, then this function returns a
/// StaticAnalysis modifying the first instruction in the sequence. Successively applying these
/// ensures that the sequence will not read from the `datum` before it has been initialized.
pub fn uninitialized<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    let Some(read) = sequence
        .iter()
        .enumerate()
        .find(|(_offs, insn)| insn.reads(datum))
    else {
        // There's no instruction in the sequence reading from `datum`
        return Ok(());
    };

    let Some(write) = sequence
        .iter()
        .enumerate()
        .find(|(_offs, insn)| insn.writes(datum))
    else {
        // There's no instruction in the sequence writing to `datum`, so `datum` is uninitialized
        // wherever it's read.
        return Err(read.1.sa());
    };

    if write.0 < read.0 {
        // The write to `datum` happened before the read, so that's okay.
        return Ok(());
    }

    Err(read.1.sa())
}

/// If the sequence writes to `datum` without reading from it afterward, then this function returns a
/// StaticAnalysis modifying the first instruction in the sequence that writes to `datum`. This
/// means that The sequence will not write to `datum` unless it is later used.
pub fn dont_expect_write<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    'outer: for (offset, write) in sequence
        .iter()
        .enumerate()
        .filter(|(_offs, insn)| insn.writes(datum))
    {
        for insn in sequence.iter().skip(offset) {
            if insn.reads(datum) {
                // The write I found is followed by a read,
                // so there's nothing to do here.
                continue 'outer;
            }

            if insn.writes(datum) {
                // The write I found gets been clobbered,
                // so let's return a StaticAnalysis
                break;
            }
        }

        return Err(write.sa().set_offset(offset));
    }
    Ok(())
}

/// If the sequence does not contains any instruction that writes to `datum`, then this returns a
/// StaticAnalysis modifying the first instruction in the sequence. Successively applying these
/// will make sure that the sequence writes to `datum`.
pub fn expect_write<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    if !sequence.iter().any(|insn| insn.writes(datum)) {
        // There's no instruction in the sequence writing to `datum`
        return Err(sequence[0].sa());
    };
    Ok(())
}

/// If the sequence does not contains any instruction that reads from `datum`, then this returns a
/// StaticAnalysis modifying the first instruction in the sequence. Successively applying these
/// will make sure that the sequence reads from `datum`.
pub fn expect_read<Datum, Insn: DataFlow<Datum>>(
    sequence: &Sequence<Insn>,
    datum: &Datum,
) -> Result<(), StaticAnalysis<Insn>> {
    if !sequence.iter().any(|insn| insn.reads(datum)) {
        // There's no instruction in the sequence writing to `datum`
        return Err(sequence[0].sa());
    };
    Ok(())
}
