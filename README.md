# strop
Superoptimizer written in Rust

I made the decision to abandon [stoc](https://github.com/omarandlorraine/stoc)
when I realized it was simply too unwieldly to work with. I needed an excuse to
learn Rust, plus I wanted a superoptimizer that could target things other than
the 6502.

### Supported architectures:
Not much here (yet). There are a few placeholders for miscellaneous
architectures, and some of the instructions have been implemented for some of
them. But I don't want to say they're *supported* as such yet. Probably the
best ones are:

- *mos6502*, because why not
- *mos65c02*, which for the time being is actually identical to *mos6502* since
  the extra opcodes are still not implemented
- *motorola6800*, it's related to the 6502s but has an extra register

### Theory of operation
The basic idea is to generate code better than what traditional optimising
compilers can do. A few of the reasons why that's possible:

- we can do an exhaustive search, while optimizing compilers generally do a
  greedy ascent. That means strop will find a global maximum, instead of a
  local maximum.

- we can put things like error margins on output variables, which can yield
  more opportunity for code optimization. That's like saying, "oh I don't care
  if the program computes things 100% correctly, so long as it's much faster",
  which I bet could have some utility.

- we can add different weights to each test case. That would be like saying,
  "oh, I don't care if the program is slower in the general case, so long as
  it's faster for these specific test cases."

(The last two are not implemented yet, but something I want to do eventually)
