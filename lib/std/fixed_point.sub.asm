; ============================================================
; Standard Library: lib/std/fixed_point.sub.asm
;
; Signed 64-bit fixed-point arithmetic with 20 fractional bits.
; Scale: 2^20 = 1048576.
; Actual value = signed stored value / 1048576.
;
; Routines consume their arguments and return results on the
; operand stack. Some temporarily use VM registers, but every
; routine restores their original values before returning.
; No routine modifies linear memory.
;
; Signed results must fit in 64 bits. Arithmetic overflow is not
; detected. Division by zero is invalid. Unless stated otherwise,
; multiplication and division truncate toward zero.
;
; Public routines use the STD__FIXED_POINT_ prefix.
; Internal labels are enclosed within double underscores.
; ============================================================

JMP __STD__FIXED_POINT_EOF__

; ============================================================
; Routine: STD__FIXED_POINT_FROM_INTEGER
;
; Convert a signed integer to fixed-point representation.
;
; Stack:
;   Before: [..., integer]
;   After:  [..., fixed_point]
;
; Preconditions: -2^43 <= integer <= 2^43 - 1.
; ============================================================

STD__FIXED_POINT_FROM_INTEGER:
	PUSH 20
	SL
	RT

; ============================================================
; Routine: STD__FIXED_POINT_TO_INTEGER_TRUNCATING
;
; Discard fractional bits, rounding toward zero.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., integer]
; ============================================================

STD__FIXED_POINT_TO_INTEGER_TRUNCATING:
	PUSH 1048576
	DIVS
	RT

; ============================================================
; Routine: STD__FIXED_POINT_FLOOR_TO_INTEGER
;
; Return the greatest integer <= the input value.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., integer]
; ============================================================

STD__FIXED_POINT_FLOOR_TO_INTEGER:
	PUSH 20
	ASR
	RT

; ============================================================
; Routine: STD__FIXED_POINT_CEIL_TO_INTEGER
;
; Return the smallest integer >= the input value.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., integer]
; ============================================================

STD__FIXED_POINT_CEIL_TO_INTEGER:
	DUP
	PUSH 20
	ASR
	SWP
	PUSH 1048575
	AND
	JZ __STD__FIXED_POINT_CEIL_DONE__
	PUSH 1
	ADD

__STD__FIXED_POINT_CEIL_DONE__:
	RT

; ============================================================
; Routine: STD__FIXED_POINT_ROUND_TO_INTEGER_NEAREST
;
; Round to the nearest signed integer; exact ties go to even.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., integer]
; ============================================================

STD__FIXED_POINT_ROUND_TO_INTEGER_NEAREST:
	; Calculate floor and the nonnegative fractional offset.
	DUP
	PUSH 20
	ASR
	SWP
	PUSH 1048575
	AND

	; Fractions greater than 1/2 round up.
	DUP
	PUSH 524288
	CGTU
	JZ __STD__FIXED_POINT_ROUND_CHECK_HALF__
	POP
	PUSH 1
	ADD
	RT

__STD__FIXED_POINT_ROUND_CHECK_HALF__:
	DUP
	PUSH 524288
	CEQ
	JZ __STD__FIXED_POINT_ROUND_DONE__

	; Exactly halfway: increment only if the floor is odd.
	POP
	DUP
	PUSH 1
	AND
	ADD
	RT

__STD__FIXED_POINT_ROUND_DONE__:
	POP
	RT

; ============================================================
; Routine: STD__FIXED_POINT_FRACTIONAL_PART
;
; Return the signed fractional part relative to truncation
; toward zero; result remains fixed-point and has magnitude < 1.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., fractional_part]
; ============================================================

STD__FIXED_POINT_FRACTIONAL_PART:
	PUSH 1048576
	REMS
	RT

; ============================================================
; Routine: STD__FIXED_POINT_ABSOLUTE
;
; Return the nonnegative magnitude of a fixed-point value.
;
; Stack:
;   Before: [..., fixed_point]
;   After:  [..., absolute_value]
;
; Precondition: input must not be -2^63, whose positive
; magnitude is not representable as a signed 64-bit integer.
; ============================================================

STD__FIXED_POINT_ABSOLUTE:
	DUP
	PUSH 0
	CLTS
	JZ __STD__FIXED_POINT_ABSOLUTE_DONE__
	PUSH 0
	SWP
	SUB

__STD__FIXED_POINT_ABSOLUTE_DONE__:
	RT

; ============================================================
; Routine: STD__FIXED_POINT_CLAMP
;
; Restrict a fixed-point value to [minimum, maximum].
;
; Stack:
;   Before: [..., value, minimum, maximum]
;   After:  [..., clamped_value]
;
; Precondition: minimum <= maximum (signed comparison).
; ============================================================

STD__FIXED_POINT_CLAMP:
	; If value < minimum, return minimum.
	PICK 2
	PICK 2
	CLTS
	JZ __STD__FIXED_POINT_CLAMP_CHECK_MAX__
	POP
	SWP
	POP
	RT

__STD__FIXED_POINT_CLAMP_CHECK_MAX__:
	; If value > maximum, return maximum.
	PICK 2
	PICK 1
	CGTS
	JZ __STD__FIXED_POINT_CLAMP_KEEP_VALUE__
	SWP
	POP
	SWP
	POP
	RT

__STD__FIXED_POINT_CLAMP_KEEP_VALUE__:
	POP
	POP
	RT

; ============================================================
; Routine: STD__FIXED_POINT_ADD
;
; Add two signed fixed-point values.
;
; Stack:
;   Before: [..., a, b]
;   After:  [..., sum]
;
; Both operands must use the same fixed-point scale.
; No scaling adjustment is necessary for addition.
;
; Preconditions:
;   The mathematical sum must fit in signed 64 bits.
;
; Overflow is not detected. An overflowing addition wraps
; modulo 2^64, potentially producing an incorrect signed
; fixed-point result.
;
; No registers or linear memory are modified.
; ============================================================

STD__FIXED_POINT_ADD:
	ADD
	RT

; ============================================================
; Routine: STD__FIXED_POINT_SUBTRACT
;
; Subtract one signed fixed-point value from another.
;
; Stack:
;   Before: [..., a, b]
;   After:  [..., difference]
;
; Calculates a - b.
;
; Both operands must use the same fixed-point scale.
; No scaling adjustment is necessary for subtraction.
;
; Preconditions:
;   The mathematical difference must fit in signed 64 bits.
;
; Overflow is not detected. An overflowing subtraction wraps
; modulo 2^64, potentially producing an incorrect signed
; fixed-point result.
;
; No registers or linear memory are modified.
; ============================================================

STD__FIXED_POINT_SUBTRACT:
	SUB
	RT

; ============================================================
; Routine: STD__FIXED_POINT_MULTIPLY
;
; Multiply two signed fixed-point values, truncating toward
; zero. Uses a 20-bit split to avoid overflowing an ordinary
; signed 64-bit intermediate product when the final result fits.
;
; Stack:
;   Before: [..., a, b]
;   After:  [..., product]
;
; Preconditions:
;   The final fixed-point product must fit in signed 64 bits.
;   Out-of-range results are not detected.
;
; Temporarily uses R0-R5; restores all their original values.
; Other registers are untouched.
; ============================================================

STD__FIXED_POINT_MULTIPLY:
	; Save working registers above the two arguments.
	RGET 0
	RGET 1
	RGET 2
	RGET 3
	RGET 4
	RGET 5

	PICK 7
	RSET 0                ; a
	PICK 6
	RSET 1                ; b

	; R2 = sign difference; R0 and R1 become unsigned magnitudes.
	RGET 0
	RGET 1
	XOR
	PUSH 63
	LSR
	RSET 2

	RGET 0
	PUSH 0
	CLTS
	JZ __STD__FIXED_POINT_MUL_A_POSITIVE__
	PUSH 0
	RGET 0
	SUB
	RSET 0

__STD__FIXED_POINT_MUL_A_POSITIVE__:
	RGET 1
	PUSH 0
	CLTS
	JZ __STD__FIXED_POINT_MUL_B_POSITIVE__
	PUSH 0
	RGET 1
	SUB
	RSET 1

__STD__FIXED_POINT_MUL_B_POSITIVE__:
	; Split each magnitude into high bits and 20 low bits.
	RGET 0
	PUSH 20
	LSR
	RSET 4                ; a_high
	RGET 1
	PUSH 20
	LSR
	RSET 5                ; b_high

	; Magnitude of result, before applying sign:
	; a_high*b_high*S + a_high*b_low + b_high*a_low
	;                       + floor(a_low*b_low/S)
	RGET 4
	RGET 5
	MUL
	PUSH 20
	SL
	RSET 3

	RGET 3
	RGET 4
	RGET 1
	PUSH 1048575
	AND
	MUL
	ADD
	RSET 3

	RGET 3
	RGET 5
	RGET 0
	PUSH 1048575
	AND
	MUL
	ADD
	RSET 3

	RGET 3
	RGET 0
	PUSH 1048575
	AND
	RGET 1
	PUSH 1048575
	AND
	MUL
	PUSH 20
	LSR
	ADD
	RSET 3

	; Negate the unsigned magnitude when the signs differ.
	RGET 2
	JZ __STD__FIXED_POINT_MUL_RESULT__
	PUSH 0
	RGET 3
	SUB
	RSET 3

__STD__FIXED_POINT_MUL_RESULT__:
	RGET 3

	; Keep the result on top while restoring saved registers.
	SWP
	RSET 5
	SWP
	RSET 4
	SWP
	RSET 3
	SWP
	RSET 2
	SWP
	RSET 1
	SWP
	RSET 0

	; Remove the two original arguments; retain the result.
	SWP
	POP
	SWP
	POP
	RT

; ============================================================
; Routine: STD__FIXED_POINT_DIVIDE
;
; Divide a by b as signed fixed-point values, truncating toward
; zero. Uses unsigned long division for the 20 fractional bits,
; avoiding an overflowing (a * 1048576) intermediate.
;
; Stack:
;   Before: [..., a, b]
;   After:  [..., quotient]
;
; Preconditions:
;   b != 0.
;   The final fixed-point quotient must fit in signed 64 bits.
;   Out-of-range results are not detected.
;
; Temporarily uses R0-R6; restores all their original values.
; Other registers are untouched.
; ============================================================

STD__FIXED_POINT_DIVIDE:
	RGET 0
	RGET 1
	RGET 2
	RGET 3
	RGET 4
	RGET 5
	RGET 6

	PICK 8
	RSET 0                ; a
	PICK 7
	RSET 1                ; b

	RGET 0
	RGET 1
	XOR
	PUSH 63
	LSR
	RSET 2                ; differing signs

	RGET 0
	PUSH 0
	CLTS
	JZ __STD__FIXED_POINT_DIV_A_POSITIVE__
	PUSH 0
	RGET 0
	SUB
	RSET 0

__STD__FIXED_POINT_DIV_A_POSITIVE__:
	RGET 1
	PUSH 0
	CLTS
	JZ __STD__FIXED_POINT_DIV_B_POSITIVE__
	PUSH 0
	RGET 1
	SUB
	RSET 1

__STD__FIXED_POINT_DIV_B_POSITIVE__:
	; Integer quotient and remainder of the magnitudes.
	RGET 0
	RGET 1
	DIVU
	RSET 3                ; quotient
	RGET 0
	RGET 1
	REMU
	RSET 4                ; remainder

	PUSH 20
	RSET 5                ; number of fractional bits

__STD__FIXED_POINT_DIV_FRACTION_LOOP__:
	RGET 5
	JZ __STD__FIXED_POINT_DIV_APPLY_SIGN__

	; Compare remainder with divisor - remainder. This avoids
	; ever computing 2*remainder if that might overflow u64.
	RGET 1
	RGET 4
	SUB
	RSET 6

	RGET 4
	RGET 6
	CGEU
	JZ __STD__FIXED_POINT_DIV_NEXT_BIT_ZERO__

	; Next binary fractional bit is one.
	RGET 4
	RGET 6
	SUB
	RSET 4
	RGET 3
	PUSH 1
	SL
	PUSH 1
	ADD
	RSET 3
	JMP __STD__FIXED_POINT_DIV_ADVANCE__

__STD__FIXED_POINT_DIV_NEXT_BIT_ZERO__:
	; Next binary fractional bit is zero.
	RGET 4
	PUSH 1
	SL
	RSET 4
	RGET 3
	PUSH 1
	SL
	RSET 3

__STD__FIXED_POINT_DIV_ADVANCE__:
	RGET 5
	PUSH 1
	SUB
	RSET 5
	JMP __STD__FIXED_POINT_DIV_FRACTION_LOOP__

__STD__FIXED_POINT_DIV_APPLY_SIGN__:
	RGET 2
	JZ __STD__FIXED_POINT_DIV_RESULT__
	PUSH 0
	RGET 3
	SUB
	RSET 3

__STD__FIXED_POINT_DIV_RESULT__:
	RGET 3

	SWP
	RSET 6
	SWP
	RSET 5
	SWP
	RSET 4
	SWP
	RSET 3
	SWP
	RSET 2
	SWP
	RSET 1
	SWP
	RSET 0

	SWP
	POP
	SWP
	POP
	RT

; ============================================================
; Routine: STD__FIXED_POINT_FROM_RATIO
;
; Convert numerator / denominator into fixed-point format,
; truncating toward zero.
;
; Stack:
;   Before: [..., numerator, denominator]
;   After:  [..., fixed_point]
;
; Preconditions: denominator != 0; result fits signed 64 bits.
; ============================================================

STD__FIXED_POINT_FROM_RATIO:
	CALL STD__FIXED_POINT_DIVIDE
	RT

; ============================================================
; Routine: STD__FIXED_POINT_SQUARE_ROOT
;
; Return floor(sqrt(value * 1048576)) in fixed-point units.
; Uses integer Newton iteration and full-range fixed division.
;
; Stack:
;   Before: [..., value]
;   After:  [..., square_root]
;
; Precondition: value >= 0 (as a signed 64-bit integer).
; The result is rounded downward to a fixed-point increment.
; No registers are modified.
; ============================================================

STD__FIXED_POINT_SQUARE_ROOT:
	DUP
	JZ __STD__FIXED_POINT_SQRT_DONE__

	; Upper bound on the root: max(value, 1.0).
	DUP
	PUSH 1048576
	CGEU
	JZ __STD__FIXED_POINT_SQRT_USE_ONE__
	DUP
	JMP __STD__FIXED_POINT_SQRT_LOOP__

__STD__FIXED_POINT_SQRT_USE_ONE__:
	PUSH 1048576

__STD__FIXED_POINT_SQRT_LOOP__:
	; Stack: [..., value, guess]
	; quotient = floor(value * 1048576 / guess).
	PICK 1
	PICK 1
	CALL STD__FIXED_POINT_DIVIDE

	; Average guess and quotient without overflowing u64:
	; (guess & quotient) + ((guess ^ quotient) >> 1).
	PICK 1
	PICK 1
	AND
	PICK 2
	PICK 2
	XOR
	PUSH 1
	LSR
	ADD

	; Discard quotient; compare next guess to old guess.
	SWP
	POP
	DUP
	PICK 2
	CGEU
	JZ __STD__FIXED_POINT_SQRT_UPDATE__

	; The decreasing iteration has converged.
	POP
	SWP
	POP
	RT

__STD__FIXED_POINT_SQRT_UPDATE__:
	; Replace old guess with next guess.
	SWP
	POP
	JMP __STD__FIXED_POINT_SQRT_LOOP__

__STD__FIXED_POINT_SQRT_DONE__:
	RT

; ============================================================
; Routine: STD__FIXED_POINT_LERP
;
; Linearly interpolate a and b using fixed-point t:
;   a*(1-t) + b*t.
;
; Stack:
;   Before: [..., a, b, t]
;   After:  [..., interpolated_value]
;
; Precondition: 0 <= t <= 1048576 (between 0 and 1).
; Each weighted product truncates toward zero. Consequently,
; the result may differ by one fixed-point unit from rounding
; the combined expression just once.
;
; Uses no registers directly. The multiplication routine uses
; working registers but restores all of their original values.
; No linear memory is modified.
; ============================================================

STD__FIXED_POINT_LERP:
	; First term: a * (1 - t).
	PICK 2
	PUSH 1048576
	PICK 2
	SUB
	CALL STD__FIXED_POINT_MULTIPLY

	; Second term: b * t.
	PICK 2
	PICK 2
	CALL STD__FIXED_POINT_MULTIPLY

	; Add the terms and remove the original arguments.
	ADD
	SWP
	POP
	SWP
	POP
	SWP
	POP
	RT

__STD__FIXED_POINT_EOF__: