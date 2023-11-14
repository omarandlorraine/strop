# library

Strop's principal purpose is to generate code, and therefore I made a simple
program to output some simple subroutines. Each subroutine is derived from a
function or closure in the source file (that is, the source is in Rust, but the
output is in ARMv4T assembly).

Each of the subroutines complies with (my understanding of) the AAPCS32 calling
convention, and so should be callable from C.

As of the time of writing, the output of the program is:

```
add:
	add r0, r0, r1
	mov pc, lr
shl:
	lsl r0, r1
	mov pc, lr
shr:
	lsl r0, r0, 0
	mov pc, lr
mul:
	mul r0, r1
	mov pc, lr
pepper:
	lsr r0, r0, 4
	lsl r0, r0, 4
	mov pc, lr
```
