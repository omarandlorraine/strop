//! Module for pruning down a search for 6502 programs.
use crate::m6502::isa::Insn;
use crate::ConstraintViolation;

/// Reduces the set of 6502 instructions that a constraint solver will consider.
#[derive(Debug, Default)]
pub struct Prune {
    readable_locations: Vec<u16>,
    writable_locations: Vec<u16>,
    indexable_locations: Vec<u16>,
    pointer_locations: Vec<u16>,
    entry_points: Vec<u16>,
}

impl Prune {
    /// Build a new, default, `Prune` object.
    pub fn new() -> Self {
        Default::default()
    }

    /// Informs the `Prune` object of a region of readable memory. The `Prune` will prune away
    /// instructions which read from locations not known to be readable.
    pub fn readable(&mut self, address: u16) -> &Self {
        self.readable_locations.push(address);
        self
    }

    /// Informs the `Prune` object of a region of writeable memory. The `Prune` will prune away
    /// instructions which write to locations not known to be writable.
    pub fn writable(&mut self, address: u16) -> &Self {
        self.writable_locations.push(address);
        self
    }

    /// Informs the `Prune` object of an array, or some other multi-byte region of memory. The
    /// `Prune` will prune away instructions which use indexed addressing modes on locations not
    /// known to be indexable.
    pub fn indexable(&mut self, address: u16) -> &Self {
        self.indexable_locations.push(address);
        self
    }

    /// Informs the `Prune` object of a pointer somewhere in memory. The `Prune` object will prune
    /// away any instructions that dereference pointers not stored at the known addresses.
    pub fn pointer(&mut self, address: u16) -> &Self {
        self.pointer_locations.push(address);
        self
    }

    /// Informs the `Prune` object of an entry point. The `Prune` will prune away any `jmp` or
    /// `jsr` instructions not targeting a known entry point.
    pub fn entry_point(&mut self, entry_point: u16) -> &Self {
        self.entry_points.push(entry_point);
        self
    }
}

fn next_in_ranges(rngs: &[u16], addr: u16) -> Option<u16> {
    rngs.iter().filter(|&&x| x > addr).min().copied()
}

impl<V: mos6502::Variant + std::clone::Clone> crate::Prune<Insn<V>> for Prune {
    fn prune(&self, i: &Insn<V>) -> ConstraintViolation<Insn<V>> {
        if let Some(addr) = i.reads_from() {
            // The instruction reads from some location.

            if self.readable_locations.is_empty() {
                // The instruction reads from a location, but there are no readable locations
                // specified. So we need to propose the next opcode
                i.next_opcode()
            } else if self.readable_locations.contains(&addr) {
                // The instruction reads from a permitted location; nothing else to check for this
                // instruction.
                ConstraintViolation::Ok
            } else if let Some(addr) = next_in_ranges(&self.readable_locations, addr) {
                // The instruction reads from the wrong location, but we know another location it
                // could try instead
                i.bump_operand(addr)
            } else {
                // The instruction reads from the wrong location and we don't know another location
                // it could try; bump it to the next opcode.
                i.next_opcode()
            }
        } else {
            ConstraintViolation::Ok
        }
    }
}

impl<V: mos6502::Variant + Default + std::clone::Clone, P: crate::Prune<Insn<V>>>
    crate::PrunedSearch<P> for Insn<V>
{
    fn first() -> Self {
        Default::default()
    }

    fn pruned_step(&mut self, prune: &P) -> bool {
        use crate::Iterable;
        loop {
            if !self.step() {
                break false;
            }
            match prune.prune(self) {
                ConstraintViolation::Ok => break true,
                ConstraintViolation::ReplaceWith(me) => *self = me,
                ConstraintViolation::Violation => break false,
            }
        }
    }
}
