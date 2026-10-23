; ============================================================
; Standard Library: lib/std/registers.sub.asm
;
; Routines for saving and restoring the values of VM registers
; using the operand stack.
;
; Supports all 16 registers or selected register ranges.
; Register ranges use an inclusive start and exclusive end.
;
; Public routines use the STD__REGISTERS_ prefix.
; Internal labels are enclosed within double underscores.
; ============================================================

JMP __STD__REGISTERS_EOF__

; ============================================================
; Routine: STD__REGISTERS_SAVE_TO_STACK_ALL
;
; Push all 16 register values onto the operand stack.
;
; Stack:
;   Before: [...]
;   After:  [..., R0, R1, ..., R15]
;
; Registers are pushed in ascending order (R0 first).
; R15 is left at the top of the stack.
;
; All registers retain their original values.
;
; Should be paired with STD__REGISTERS_RESTORE_FROM_STACK_ALL
; when the saved values are no longer needed.
; ============================================================

STD__REGISTERS_SAVE_TO_STACK_ALL:
	RGET 0
	RGET 1
	RGET 2
	RGET 3
	RGET 4
	RGET 5
	RGET 6
	RGET 7
	RGET 8
	RGET 9
	RGET 10
	RGET 11
	RGET 12
	RGET 13
	RGET 14
	RGET 15
	RT

; ============================================================
; Routine: STD__REGISTERS_RESTORE_FROM_STACK_ALL
;
; Pop 16 values from the operand stack and restore R0-R15.
;
; Stack:
;   Before: [..., R0, R1, ..., R15]
;   After:  [...]
;
; Registers are restored in descending order (R15 first).
;
; The saved register values must be at the top of the
; operand stack in the original order.
;
; Any temporary values pushed after saving the registers
; must be removed before calling this routine.
;
; Any changes to register values since saving are overwritten.
; ============================================================

STD__REGISTERS_RESTORE_FROM_STACK_ALL:
	RSET 15
	RSET 14
	RSET 13
	RSET 12
	RSET 11
	RSET 10
	RSET 9
	RSET 8
	RSET 7
	RSET 6
	RSET 5
	RSET 4
	RSET 3
	RSET 2
	RSET 1
	RSET 0
	RT

; ============================================================
; Routine: STD__REGISTERS_SAVE_TO_STACK_FROM_START_TO_END
;
; Push the values of registers in [start, end) onto the
; operand stack, in ascending register order.
;
; Stack:
;   Before: [..., start, end]
;   After:  [..., R(start), ..., R(end - 1)]
;
; Valid register boundaries are 0 through 16.
; If start >= end, no register values are pushed.
;
; No registers are modified.
;
; The internal loop state is packed into one word:
;   Bits 0-7:   Current register index
;   Bits 8-15:  Exclusive end index
;
; This allows saved values to accumulate beneath the
; loop state without using temporary registers.
; ============================================================

STD__REGISTERS_SAVE_TO_STACK_FROM_START_TO_END:
	; Pack start and end:
	; state = (end << 8) | start.
	PUSH 8
	SL
	OR

__STD__REGISTERS_SAVE_RANGE_LOOP__:
	; Extract the current register index.
	DUP
	PUSH 255
	AND

	; Extract the exclusive end index.
	PICK 1
	PUSH 8
	LSR

	; Continue while current < end.
	CLTU
	JZ __STD__REGISTERS_SAVE_RANGE_DONE__

	; Read the current register indirectly.
	DUP
	PUSH 255
	AND
	RGET

	; Keep the saved value beneath the loop state.
	SWP

	; Advance to the next register.
	PUSH 1
	ADD

	JMP __STD__REGISTERS_SAVE_RANGE_LOOP__

__STD__REGISTERS_SAVE_RANGE_DONE__:
	; Remove the loop state, leaving saved values.
	POP
	RT

; ============================================================
; Routine: STD__REGISTERS_RESTORE_FROM_STACK_FROM_START_TO_END
;
; Restore the values of registers in [start, end) using
; values previously saved on the operand stack.
;
; Stack:
;   Before: [..., R(start), ..., R(end - 1), start, end]
;   After:  [...]
;
; Valid register boundaries are 0 through 16.
; If start >= end, no registers are restored.
;
; Registers are restored in descending order.
;
; The saved register values must remain in their original
; order beneath the start and end arguments.
;
; Any modifications to restored registers are overwritten.
;
; The internal loop state is packed into one word:
;   Bits 0-7:   Inclusive start index
;   Bits 8-15:  Current exclusive end index
; ============================================================

STD__REGISTERS_RESTORE_FROM_STACK_FROM_START_TO_END:
	; Pack start and end:
	; state = (end << 8) | start.
	PUSH 8
	SL
	OR

__STD__REGISTERS_RESTORE_RANGE_LOOP__:
	; Extract the current exclusive end.
	DUP
	PUSH 8
	LSR

	; Extract the inclusive start.
	PICK 1
	PUSH 255
	AND

	; Continue while end > start.
	CGTU
	JZ __STD__REGISTERS_RESTORE_RANGE_DONE__

	; Move the topmost saved value above the loop state.
	SWP

	; Calculate the register index: end - 1.
	PICK 1
	PUSH 8
	LSR
	PUSH 1
	SUB

	; Restore that register indirectly.
	RSET

	; Decrement the exclusive end index.
	PUSH 256
	SUB

	JMP __STD__REGISTERS_RESTORE_RANGE_LOOP__

__STD__REGISTERS_RESTORE_RANGE_DONE__:
	; Remove the loop state.
	POP
	RT

; ============================================================
; Routine: STD__REGISTERS_SAVE_TO_STACK_FROM_ZERO_TO_END
;
; Push the values of registers in [0, end) onto the stack.
;
; Stack:
;   Before: [..., end]
;   After:  [..., R0, ..., R(end - 1)]
;
; The valid end range is 0 through 16.
; No registers are modified.
; ============================================================

STD__REGISTERS_SAVE_TO_STACK_FROM_ZERO_TO_END:
	PUSH 0
	SWP
	CALL STD__REGISTERS_SAVE_TO_STACK_FROM_START_TO_END
	RT

; ============================================================
; Routine: STD__REGISTERS_RESTORE_FROM_STACK_FROM_ZERO_TO_END
;
; Restore the values of registers in [0, end).
;
; Stack:
;   Before: [..., R0, ..., R(end - 1), end]
;   After:  [...]
;
; The valid end range is 0 through 16.
; Registers are restored in descending order.
; ============================================================

STD__REGISTERS_RESTORE_FROM_STACK_FROM_ZERO_TO_END:
	PUSH 0
	SWP
	CALL STD__REGISTERS_RESTORE_FROM_STACK_FROM_START_TO_END
	RT

__STD__REGISTERS_EOF__: