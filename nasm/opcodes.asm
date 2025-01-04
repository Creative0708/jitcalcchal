BITS 64
; This NASM file is used to derive the instruction encoding in the Rust program.

; reset
xor rax, rax

; + opcode
inc rax

; - opcode
dec rax

; * opcode
shl rax, 1

; / opcode
; (this is just what compiler explorer generated)
; https://rust.godbolt.org/z/K4f3oaozs
mov rbx, rax
shr rax, 63
add rax, rbx
sar rax, 1

; return
ret
