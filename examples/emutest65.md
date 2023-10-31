# emutest65

Strop includes several emulators for the 6502. This program does a brute-force
search for programs that behave differently on the two emulators. This is
intended to find bugs, and has already found
[one](https://github.com/mre/mos6502/pull/92).

To find these programs exposing bugs, emutest65 goes through *all* 6502
programs which are not longer than 5 instructions, do not do any flow control
(jumps, returns, conditional branches etc. are excluded), and which do not
touch memory. It is assumed that most logic errors will exercise the ALU only,
and so can be found using implied-mode and immediate-mode instructions, which
keeps the run-time down significantly.
