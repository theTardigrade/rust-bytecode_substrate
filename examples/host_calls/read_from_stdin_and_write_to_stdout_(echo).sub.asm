; Allocate 256 bytes of linear memory.
PUSH 256
MEMGRW
POP

; Read up to 256 bytes from stdin.
PUSH 0
PUSH 256
HOST RDIN

; Stack now contains [bytes_read].

; Put the address before the length.
PUSH 0
SWP

; Write the bytes we received to stdout.
HOST WROUT