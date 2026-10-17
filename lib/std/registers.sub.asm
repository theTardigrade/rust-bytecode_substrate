; ============================================================
; Standard Library: lib/std/registers.sub.asm
;
; Routines for saving and restoring the values of all 16
; VM registers using the operand stack.
;
; Public routines use the STD__REGISTERS_ prefix.
; Internal labels are enclosed within double underscores.
; ============================================================

JMP __STD__REGISTERS_EOF__

; ============================================================
; Routine: STD__REGISTERS_SAVE_ALL_TO_STACK
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
; Should be paired with STD__REGISTERS_RESTORE_ALL_FROM_STACK
; when the saved values are no longer needed.
; ============================================================

STD__REGISTERS_SAVE_ALL_TO_STACK:
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
; Routine: STD__REGISTERS_RESTORE_ALL_FROM_STACK
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

STD__REGISTERS_RESTORE_ALL_FROM_STACK:
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

__STD__REGISTERS_EOF__: