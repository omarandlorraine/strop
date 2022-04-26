# strop
[![Build Status](https://github.com/omarandlorraine/strop/workflows/Rust/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=Rust)

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

### Supported instruction sets:
I don't want to say they're *supported* as such yet: many instructions are still
missing, which will result in suboptimal programs being generated for some
functions, or programs not being found where they could be. Notably, some
control flow instructions are completely absent from strop. But probably the
best instruction sets so far are:

 - 8080, *the famous Intel chip*
 - kr580vm1, *a Ukrainian 8080 variant*
 - z80, *the famous Z80, very popular with retrocomputing enthusiasts*
 - sm83, *weirdo found in certain nintendos*
 - 2a03, *another weirdo found in certain nintendos; this is a 6502 with no decimal mode*
 - 6502, *famous 8-bitter from the 1970s*
 - 65i02, *6502 core instruction set plus some illegal opcodes*
 - 65c02, *6502 plus CMOS extensions*
 - 6800, *Motorola 8-bitter*
 - 6801, *successor to the 6800*
 - pic12, *low-end microcontroller from Microchip*
 - pic14, *extension to PIC12*
 - pic16, *extension to PIC12*
 - stm8, *microcontroller that's similar to a beefed-up 6502*

I've tried to pick ones I use or like, and then I've added the low-hanging
fruit like their relatives and so on. I've also tried to make this extensible,
and easy to add whatever architectures in the future.

### Theory of operation
The basic idea is to generate code better than what traditional optimising
compilers can do. A few of the reasons why that's possible:

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
  predictability and such likes, and optimize for the same.

(The last three are not implemented yet, but something I want to do eventually)

How are we going to do this? The way strop generates code is by running a code
sequence against a set of test cases (these may be generated by strop itself or
supplied by the user). The code is mutated and run against the test cases over
and over again. When the test cases all pass, we know the program is good. As
the code is run, we can analyse it for characteristics like speed and size, and
this information can be fed into the way we mutate or select the code sequence.

### Some example runs

What if we want to multiply some number by a constant? For this example, the
number is in register B, the constant is 15, and the output is in register A.
So you would run:

    strop --arch 6800 --function mult15 --in b --out a

A couple of seconds later, the program outputs:

    	tba
    	aba
    	aba
    	tab
    	aba
    	asla
    	aba

Since the Motorola 6800 has no multiply instruction, it's generated some shifts
and adds and things that implement a multiplication by 15.

You might need something other than the miscellaneous built-in functions that
I've decided to put in. You might want to define your own functions. If you can
generate an appropriate JSON file, you can pass it to strop and have strop
generate the code that satisfies all test cases in the file. See the
`examples/` folder for examples. 

    strop --arch 6800 -f examples/decimal_adjust.json

produces the following code,

    	add #0
    	daa

As to why the `add #0` was generated, my guess is that `daa` depends on the
state of certain flags, and `add #0` sets these flags right.
