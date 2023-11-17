# library

Strop's principal purpose is to generate code, and therefore I made a simple
program to output some simple subroutines. Each subroutine is derived from a
function or closure in the source file (that is, the source is in Rust, but the
output is in ARMv4T assembly).

Each of the subroutines complies with (my understanding of) the AAPCS32 calling
convention, and so should be callable from C.

As of the time of writing, the output of the program is:

```rust
add:
	add r0, r0, r1   ; 0x1808
	mov pc, lr
shl:
	lsl r0, r1     ; 0x4088
	mov pc, lr
shr:
	asr r0, r0, 0     ; 0x1000
	mov pc, lr
mul:
	mul r0, r1     ; 0x4348
	mov pc, lr
salt:
	mov r3, #224     ; 0x2be0
	sub r6, #53     ; 0x3e35
	mul r0, r1     ; 0x4348
	ldmia r3!, {r1, }     ; 0xc302
	bvc -14     ; 0xd9f2
	stmia r7!, {r2, r3, r5, r6, r7, }     ; 0xcf6c
	bmi -30     ; 0xd6e2
	cmp r7, #152     ; 0x2798
	bcc -52     ; 0xd3cc
	ldmia r1!, {r0, r2, r5, r6, r7, }     ; 0xc165
	lsr r1, r6, 18     ; 0x0c8e
	ldmia r2!, {r6, r7, }     ; 0xc240
	lsl r6, r2, 31     ; 0x07f2
	ldmia r6!, {r2, r6, r7, }     ; 0xc6c4
	mov r0, #42     ; 0x282a
	bge -10     ; 0xdcf6
	pop {r2, r3, r4, r5, }     ; 0xbcbc
	bl 875     ; 0xfb6b
	add r8, r5     ; 0x4445
	asr r0, r3, 28     ; 0x1703
	mov pc, lr
pepper:
	stmia r7!, {r3, r4, r5, }     ; 0xcf38
	add r7, #26     ; 0x371a
	sub r7, #27     ; 0x3f1b
	bic r0, r7     ; 0x43b8
	add r7, r0, #2     ; 0x1d57
	cmp r6, #185     ; 0x26b9
	b 2041     ; 0xe7f9
	mov r7, #139     ; 0x2f8b
	mov pc, lr
```
