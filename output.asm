global _start
section .text
_start:
    mov rax, 60     ; sys_exit
    mov rdi, 67    ; exit code
    syscall
