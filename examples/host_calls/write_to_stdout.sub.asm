; Allocate 6 bytes of linear memory.
PUSH 6
MEMGROW
POP

; Store "Hello\n" in memory.

PUSH 0
PUSH 72      ; H
MEMST8

PUSH 1
PUSH 101     ; e
MEMST8

PUSH 2
PUSH 108     ; l
MEMST8

PUSH 3
PUSH 108     ; l
MEMST8

PUSH 4
PUSH 111     ; o
MEMST8

PUSH 5
PUSH 10      ; newline
MEMST8

; Write memory[0..6] to stdout.
PUSH 0       ; address
PUSH 6       ; length
HOSTCALL 0

HALT