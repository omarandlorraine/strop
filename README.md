# strop
Superoptimizer written in Rust

I made the decision to abandon [stoc](https://github.com/omarandlorraine/stoc)
when I realized it was simply too unwieldly to work with. I needed an excuse to
learn Rust, plus I wanted a superoptimizer that could target things other than
the 6502., So, strop was born, the *st*ochastic *op*timizer, written in *R*ust.

Okay, okay, it's not stochastic (yet). Like very early versions of stoc, it has
an exhaustive search only. Some of the warnings at build time about functions
that never get used, is because the stochastic search is not implemented yet.

### Supported architectures:
Not much here (yet). There are a few placeholders for miscellaneous
architectures, and some of the instructions have been implemented for some of
them. But I don't want to say they're *supported* as such yet. Probably the
best ones are:

- *mos6502*, because why not
- *mos65c02*, which has all the same instructions as mos6502 plus some extras
- *motorola6800*, it's related to the 6502s but has an extra register and some
  other goodies

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
  "oh, I don't care if the program is slower in the general case, so long as
  it's faster for these specific test cases."

(The last two are not implemented yet, but something I want to do eventually)

### Some example runs

What if we want to multiply some number by a constant? For this example, the
number is in register B, the constant is 15, and the output is in register A.
So you'd run:

    strop --arch motorola6800 --function mult15 --search exh --live-in b --live-out a

As you can see, `mult15` was used for the function name. Any positive integer
can go here actually. But the `mult*n*` functions are not defined for integer
overflow. So for example `mult129` is probably not what it seems at first
glance.

And the program outputs:

	tba
	aba
	aba
	asla
	aba
	asla
	aba

Or let's say you want a multiply by three routine for the 6502. So you run

    target/debug/strop --arch mos6502 --function mult5 --search exh --live-in a --live-out a

Okay, the program spits out the following:

	sta 3
	asl a
	asl a
	adc 3

So that's store the original accumulator in zero page location 3, multiply the
accumulator by four in the obvious way, and then add the original value. I
don't yet know why location 3, or why the carry flag wasn't cleared anywhere.
That's a bug.

This was found by an exhaustive search. The difficulty is that this takes a
long time to run, and the runtime is only going to get worse as I add more
instructions to each architecture. Eventually there will also be miscellaneous
stochastic search strategies to mitigate this problem.
