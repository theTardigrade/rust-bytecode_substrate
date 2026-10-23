; ============================================================
; Standard Library: lib/std/iteration.sub.asm
;
; Routines for repeating a callback over an unsigned integer
; range, with an inclusive start and exclusive end.
;
; The callback receives the current iteration value on the
; operand stack. It must consume that value and leave all
; underlying stack values unchanged before returning.
;
; These routines do not modify any VM registers directly.
; Any register changes made by callbacks remain in effect.
;
; Public routines use the STD__ITERATION_ prefix.
; Internal labels are enclosed within double underscores.
; ============================================================

JMP __STD__ITERATION_EOF__

; ============================================================
; Routine: STD__ITERATION_FROM_ZERO_TO_END
;
; Invoke a callback for each integer from 0 to end - 1.
;
; Stack:
;   Before: [..., end, callback]
;   After:  [...]
;
; The callback is an absolute bytecode address.
; ============================================================

STD__ITERATION_FROM_ZERO_TO_END:
	PUSH 0
	CALL __STD__ITERATION_RUN__
	RT

; ============================================================
; Routine: STD__ITERATION_FROM_START_TO_END
;
; Invoke a callback for each integer from start to end - 1.
;
; Stack:
;   Before: [..., start, end, callback]
;   After:  [...]
;
; If start >= end, the callback is never invoked.
; ============================================================

STD__ITERATION_FROM_START_TO_END:
	; Duplicate the starting value as the loop counter.
	PICK 2

	CALL __STD__ITERATION_RUN__

	; Remove the original start value.
	POP
	RT

; ============================================================
; Internal routine: __STD__ITERATION_RUN__
;
; Stack:
;   Before: [..., end, callback, current]
;   After:  [...]
;
; The three values are maintained on the operand stack,
; so no registers are needed for loop state.
; ============================================================

__STD__ITERATION_RUN__:

__STD__ITERATION_RUN_LOOP__:
	; Test current < end.
	DUP
	PICK 3
	CLTU
	JZ __STD__ITERATION_RUN_DONE__

	; Pass a copy of the current value to the callback.
	DUP

	; Duplicate the callback address.
	PICK 2

	; Indirectly call the callback.
	CALL

	; Advance the current value.
	PUSH 1
	ADD

	JMP __STD__ITERATION_RUN_LOOP__

__STD__ITERATION_RUN_DONE__:
	; Discard current, callback, and end.
	POP
	POP
	POP
	RT

__STD__ITERATION_EOF__: