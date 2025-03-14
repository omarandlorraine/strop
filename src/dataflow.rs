//! This module contains miscellaneous conveniences for performing dataflow analysis on code
//! sequences.

use crate::IterationResult;

pub trait DataFlow<T> {
    //! A trait for very local dataflow. It's generic across `T`, a type intended to represent
    //! "things" a machine instruction may read from or write to.
    //!
    //! For example, a type representing a Z80 machine code instruction could implement this for
    //! the Z80's register file, the flags, the I/O space and the address space.

    /// returns true iff the variable `t` is read (used) by the instruction or basic block before
    /// any assignment. Such a variables must be live at the start of the block.
    fn reads(&self, t: &T) -> bool;

    /// returns true iff the variable `t` is assigned (written to) by the instruction or basic
    /// block, effectively "killing" any previous value it held.
    fn writes(&self, t: &T) -> bool;

    /// Modifies the instruction
    fn modify(&mut self) -> IterationResult;

    /// Modifies the instruction so that it reads from `t`.
    fn make_read(&mut self, t: &T) -> IterationResult {
        while !self.reads(t) {
            self.modify()?;
        }
        Ok(())
    }

    /// Modifies the instruction so that it writes to `t`.
    fn make_write(&mut self, t: &T) -> IterationResult {
        while !self.writes(t) {
            self.modify()?;
        }
        Ok(())
    }

    /// Modifies the instruction so that it does not read from `t` without having written to `t`
    /// first.
    fn not_live_in(&mut self, t: &T) -> IterationResult {
        if !self.writes(t) {
            while self.reads(t) {
                self.modify()?;
            }
        }
        Ok(())
    }

    /// Modifies the instruction so that it does not write to `t`.
    fn not_live_out(&mut self, t: &T) -> IterationResult {
        while self.writes(t) {
            self.modify()?;
        }
        Ok(())
    }
}
