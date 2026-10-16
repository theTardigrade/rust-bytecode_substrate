; Allocate 256 bytes of linear memory.
PUSH 256
MEMGROW
POP

; Read up to 256 bytes from stdin.
PUSH 0
PUSH 256
HOSTCALL RDIN

; Stack now contains [bytes_read].

; Put the address before the length.
PUSH 0
SWAP

; Write the bytes we received to stdout.
HOSTCALL WROUT