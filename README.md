# strop
[![Build Status](https://github.com/omarandlorraine/strop/workflows/Rust/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=Rust)
[![crates.io](https://img.shields.io/crates/v/strop)](https://crates.io/crates/strop)

# Introduction to the project

The project is a superoptimizer written in Rust. Unlike the optimizers in a
compiler's backend, a superoptimizer generates or optimizes machine code
programs by some kind of search strategy that does not take into account the
program that's being generated. In particular, strop uses stochastic searches
to generate or optimize programs.

I abandoned [stoc](https://github.com/omarandlorraine/stoc), a similar thing
done in C, when I realized it was simply too unwieldly to work with. I needed
an excuse to learn Rust, plus I wanted a superoptimizer that could target
things other than the 6502, So, strop was born, the *st*ochastic *op*timizer,
written in *R*ust.

Strop currently supports these instruction sets, as indicated by the status of
the badges:

 * [![Build Status](https://github.com/omarandlorraine/strop/workflows/mos6502/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=mos6502)

A few of the reasons why it's possible to generate better code than a compiler
does:

- we can do an exhaustive search, while optimizing compilers generally do a
  greedy ascent. That means strop will find a global maximum, instead of a
  local maximum.

- we can put things like error margins, and don't-care bits on output
  variables, which can yield more opportunity for code optimization. That's
  like saying, "oh I don't care if the program computes things 100% correctly,
  so long as it's much faster", which I bet could have some utility.

- we can add different weights to each test case. That would be like saying,
  "oh, I don't care if the program is suboptimal in the general case, so long as
  it's more optimal for these specific test cases."

- having run all the test cases, we can take some measurements such as branch
  predictability, interrupt latency, and such likes, and optimize for the same.

(Some of these are not implemented yet, but are something I want to do
eventually)

# Theory of operation

To generate a machine instruction, we just use a random-number generator, and
check if the random numbers encode a valid instruction. If not, we just get the
next numbers from the RNG. See the `Instruction::new` method for
implementations.

A `Snippet` is an ordered list of instructions for some architecture, and may
be tested for such properties as whether or not it contains any loops,
conditional branches, subroutine calls, or accesses to the stack. Its
`to_bytes` method returns an iterator across the bytes encoding the program.

As a heuristic measure, a snippet also can be tested for closeness to the state
transition probabilities of a corpus of existing programs (this is a quick and
dirty way of checking for similarity to existing software; it could disqualify,
for example, a snippet beginning with a return-from-subroutine instruction,
since that's not commonly observed in the wild).

An emulator, (and currently exactly one emulator is included here,
[mos6502](https://github.com/mre/mos6502)), may be used the run the snippet.
For instance, if we are interested in snippets which add two numbers together
and stores the result somewhere, one could load the emulator's state with a
predetermined value for each of the addends in the expected location, and
checks that the program stores the correct result in the expected location. 
