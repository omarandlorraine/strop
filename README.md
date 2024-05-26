# strop
[![Build Status](https://github.com/omarandlorraine/strop/workflows/Rust/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=Rust)
[![crates.io](https://img.shields.io/crates/v/strop)](https://crates.io/crates/strop)

strop, the *st*ochastic *op*timizer, written in *R*ust.

Like a compiler, strop generates assembly that computes a given function. But
unlike a compiler, it generates assembly-language subroutines by a random
search or a bruteforce search.

### Examples

If you're wondering what this could be used for, here is an example:

* [arpa_inet_h](examples/arpa_inet_h.md), which emits the functions in
  `arpa/inet.h` in Z80 assembler.

### Supported instruction sets:

Strop currently has a relatively small number of backends:

 * **armv4t**, which targets the Thumb instruction set as found on the ARM7TDMI.
 * **mos6502**, which targets the
   [6502](https://en.wikipedia.org/wiki/MOS_Technology_6502). This backend is
accompanied by emulators for some of the different variant 6502s.
 * **z80**, which targets the [Z80](https://en.wikipedia.org/wiki/Zilog_Z80),
   another retro eight-bitter.

### Static analysis:

There are also static analysis passes, which can for example:
 * exclude any instruction from this or that instruction set extension (for
example, the mos6502 backend can use these to produce code compatible with
_all_ models, or _several_ models of the 6502)
 * exclude any program which contains a conditional branch (could be useful if
you want to alter a routine suffering from branch mispredictions)
 * exclude any program which accesses memory outside of allowed ranges
 * exclude any program which is not a well-formed subroutine

These static analysis passes have the added benefit, that they can
drastically reduce the search space, thus speeding up the execution time of
strop itself.

### Theory of operation

Strop currently combines two main search strategies, the Stochastic search and
the Bruteforce search, with various static analysis passes which guide the
search and restrict the search space. The search algorithms themselves, and the
static analysis passes, all implement the same traits, so that they may be
combined at will by the program which uses the library strop. These objects
also all have the `.iter()` method, so that client code can idiomatically
iterate across the programs visited by the search algorithm. This is
illustrated by the example below:

```rust
for program in
	Z80Instruction::stochastic_search()        // stochastic search for Z80 programs,
   .compatibility(Intel8080)                   // that are compatible with the Intel 8080,
   .linkage(Subroutine)                        // and are well-formed subroutines.
   .iter()                                     // (Rust idiom for instantiating an iterator)
```

Of course, the creation of such a search is not limited to the `Z80Instruction`
type, but is generic across all types implementing the `Instruction` trait, and
strop includes a few different ones, as listed above.

