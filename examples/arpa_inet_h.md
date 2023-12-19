# arpa_inet_h

Strop's principal purpose is to generate code, and therefore I made a simple
program to output some simple subroutines. These subroutines use Z88dk's FASTCALL
calling convention, and so ought to be callable from C using the proper definitions.

As of the time of writing, one possible output of the program is:

```asm
thread 'main' panicked at examples/arpa_inet_h.rs:35:10:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

... because strop does not find any solutions. Not sure why yet.
