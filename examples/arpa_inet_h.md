# arpa_inet_h

Strop's principal purpose is to generate code, and therefore I made a simple
program to output some simple subroutines. These subroutines use Z88dk's FASTCALL
calling convention, and so ought to be callable from C using the proper definitions.

As of the time of writing, one possible output of the program is ...

```asm
htons:
	EX DE, HL
	RET
htonl:
ntohl:
	RET
ntohs:
	LD E, H
	LD D, E
	LD E, L
	RET
```

... which appears to do some kind of endianness conversion for the `*s`
functions at least.

