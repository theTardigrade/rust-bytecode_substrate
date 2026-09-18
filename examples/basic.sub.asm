; Demonstrates memory, arithmetic, comparisons, bitwise operations,
; shifts, direct and indirect calls, labels, branches and comments.
;
; Final stack: [42]

NOOP

; Allocate 16 bytes of linear memory.
PUSH 16
MEMGROW
POP

; Check that memory is now 16 bytes long.
MEMSIZE
PUSH 16
CEQ
JZ failure

; Fill addresses 4..7 with the byte value 7.
PUSH 4
PUSH 7
PUSH 4
MEMFILL8

; Verify one of the filled bytes.
PUSH 4
MEMLD8
PUSH 7
CEQ
JZ failure

; Store 21 at addresses 0 and 1.
PUSH 0
PUSH 21
MEMST8

PUSH 1
PUSH 21
MEMST8

; Copy those two bytes to addresses 2 and 3.
PUSH 2
PUSH 0
PUSH 2
MEMCOPY8

; Load the copied values and add them: 21 + 21 = 42.
PUSH 2
MEMLD8

PUSH 3
MEMLD8

ADD

; Exercise ordinary relative CALL/RET while preserving 42.
CALL arithmetic_test

; Exercise an absolute indirect call.
PADDR bitwise_test
CALLIND

; A couple of signed-operation checks.
PUSH -20
PUSH 6
DIVS
PUSH -3
CEQ
JZ failure

PUSH -20
PUSH 6
REMS
PUSH -2
CEQ
JZ failure

; Check the value we have been carrying through the program.
DUP
PUSH 42
CEQ
JZ failure

JMP finish


; Exercise arithmetic and stack operations without changing
; the value passed to this function.
arithmetic_test:
	PUSH 2
	MUL                 ; 42 * 2 = 84

	PUSH 2
	DIVU                ; 84 / 2 = 42

	DUP
	PUSH 5
	REMU                ; 42 % 5 = 2
	POP                ; discard the remainder

	PUSH 0
	SWAP
	ADD                 ; 0 + 42 = 42

	RET


; Exercise bitwise operations and shifts while preserving 42.
bitwise_test:
	PUSH 0
	OR                  ; 42 | 0 = 42

	PUSH -1
	AND                 ; 42 & all-one bits = 42

	PUSH 0
	XOR                 ; 42 ^ 0 = 42

	NOT
	NOT                 ; double inversion restores 42

	PUSH 1
	SL                  ; 42 << 1 = 84

	PUSH 1
	LSR                 ; 84 >> 1 = 42

	PUSH 1
	ASR                 ; 42 >> 1 = 21

	PUSH 1
	SL                  ; 21 << 1 = 42

	RET


failure:
	; Reaching here means one of the checks failed.
	HALT

finish:
	HALT
