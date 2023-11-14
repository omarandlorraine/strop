# strop
[![Build Status](https://github.com/omarandlorraine/strop/workflows/Rust/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=Rust)
[![crates.io](https://img.shields.io/crates/v/strop)](https://crates.io/crates/strop)

Superoptimizer written in Rust

This program stochastically generates assembly language programs that compute a
given function. The idea is you give it a function to compute and specify which
registers and things to use, and strop will generate and output a pretty good
program which does the specified thing.

I abandoned [stoc](https://github.com/omarandlorraine/stoc), a similar thing
done in C, when I realized it was simply too unwieldly to work with. I needed
an excuse to learn Rust, plus I wanted a superoptimizer that could target
things other than the 6502, So, strop was born, the *st*ochastic *op*timizer,
written in *R*ust.

### Examples

If you're wondering what this could be used for, here are some examples:

* [emutest65](examples/emutest65.md), which searches for programs behaving
  differently on different emulators, exposing bugs
* [library](examples/library.md), which produces a small library of
  miscellaneous functions, in Thumb assembly, all callable from C.

### Supported instruction sets:

Strop currently has a relatively small number of backends:

 * **mos6502** which targets the
   [6502](https://en.wikipedia.org/wiki/MOS_Technology_6502). This backend is
accompanied by emulators for some of the different variant 6502s.
 * **armv4t**, which targets the Thumb instruction set as found on the ARM7TDMI.

To add a backend should be a relatively simple task:

1. Select an emulator that will execute the necessary instructions,
2. Make a type representing an instance of such an instruction, and `impl Instruction`
for that type.
3. Make a type representing an instance of the emulator's state, and `impl Emulator`
for that type.

### Static analysis:
There are also static analysis passes, which can for example:
 * exclude any instruction from this or that instruction set extension (for
example, the robo6502 backend can use these to produce code compatible with
_all_ models, or _several_ models of the 6502)
 * exclude any program which contains a conditional branch (could be useful if
you want to alter a routine suffering from branch mispredictions)
 * exclude any program which accesses memory outside of allowed ranges

These static analysis passes have the added benefit, that they can
drastically reduce the search space, thus speeding up the execution time of
strop itself.

### Theory of operation

Strop currently includes two main search strategies, the Stochastic search and
the Bruteforce search.

* The Bruteforce search simply iterates over all possible programs. Being an
iterator, it easily and idiomatically combines with filters like those provided
by the static analysis passes, to yield only those programs which are of
interest.

* The Stochastic search makes random changes to a program, and uses an error
function to guide the search towards those programs which are of interest. The
error function may penalize suboptimal programs, and programs producing
erroneous output. A Stochastic search also makes sure to propose only programs
which pass the static analysis.

