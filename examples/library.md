# library

Strop's principal purpose is to generate code, and therefore I made a simple
program to output some simple subroutines. Each subroutine is derived from a
function or closure in the source file (that is, the source is in Rust, but the
output is in ARMv4T assembly).

Each of the subroutines complies with (my understanding of) the AAPCS32 calling
convention, and so should be callable from C. No attempt has been made at
verifying or testing this.

As of the time of writing, one possible output of the program is:

```assembler
add:
	add r0, r0, r1   ; 0x1808
	mov pc, lr
shl:
	lsl r0, r1     ; 0x4088
	mov pc, lr
shr:
	asr r0, r1     ; 0x4108
	mov pc, lr
mul:
	mul r0, r1     ; 0x4348
	mov pc, lr
salt:
	adc r0, r0     ; 0x4140
	cmn r7, r0     ; 0x42c7
	adc r0, r1     ; 0x4148
	mov pc, lr
pepper:
	sub r5, #28     ; 0x3d1c
	ldmia r5!, {r0, r2, r5, }     ; 0xc525
	and r0, r5     ; 0x4028
	mov pc, lr
```

The first four subroutines, `add`, `shl`, `shr`, and `mul`, all of which are as
short as a single instruction, were found by way of bruteforce search. The
other two, `salt` and `pepper`, were found by the stochastic search.
