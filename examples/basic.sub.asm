; Demonstrates arithmetic, functions, conditional jumps,
; unconditional jumps, labels and comments.
;
; Final stack: [42]

PUSH 5
CALL add_ten        ; 5 + 10 = 15

PUSH 3
SUB                 ; 15 - 3 = 12

PUSH 0
JZ was_zero         ; zero is popped, so this branch is taken

; This code should be skipped.
PUSH 100
JMP finish

was_zero:
	CALL add_thirty  ; 12 + 30 = 42

finish:
	HALT

; Add 10 to the value currently on top of the stack.
add_ten:
	PUSH 10
	ADD
	RET

; Add 30 to the value currently on top of the stack.
add_thirty:
	PUSH 30
	ADD
	RET