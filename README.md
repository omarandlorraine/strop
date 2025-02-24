# strop
[![Build Status](https://github.com/omarandlorraine/strop/actions/workflows/tier1.yml/badge.svg?branch=master)](https://github.com/omarandlorraine/strop/actions?workflow=Checks)
[![crates.io](https://img.shields.io/crates/v/strop)](https://crates.io/crates/strop)

strop, the *st*ochastic *op*timizer, written in *R*ust.

Like a compiler, strop generates assembly that computes a given function. But
unlike a compiler, it generates assembly-language subroutines by a random
search or a brute-force search.

### Ancillary documents:

 * [LICENSE](LICENSE.md) It's just the MIT license.
 * [Theory of operation](THEORY_OF_OPERATION.md) High-level documentation explaining how strop works.

### What it's for

To see what strop could be used for:

 * [opt](examples/opt.rs) optimizes an existing machine code function
 * [gen](examples/gen.rs) generates a Z80 function matching the Rust function, after a fashion compiling Rust to Z80.
 * [peephole](examples/peephole.rs) discovers lacunae in the peephole optimizers and other constraints

### Supported instruction sets:

Strop currently has the following back-ends:

 * **armv4t**, which targets the ARMv4T processors, such as the ARM7TDMI
 * **m68000**, which targets the Motorola 68000
    * NB. This back-end is gated by the `m68k` feature since it requires nightly Rust
 * **m6502**, targets various models of the MOS 6502
    * Supports the NMOS and CMOS variants and others, thanks to the
      [mos6502](https://github.com/mre/mos6502) dependency.
 * **m6809**, which targets the Motorola 6809
 * **z80**, which targets the Zilog Z80

