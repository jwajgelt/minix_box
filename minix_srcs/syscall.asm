hello:
dq "hello"

global _start
_start:
mov ebx, 1  ; fd = stdout
mov ecx, hello  ; buf = hello
mov edx, 5  ; count = 5
int 3       ; trap to kernel, to be caught in test
mov eax, 1  ; system call number (sys_exit)
mov ebx, 0  ; exit code 0
int 0x80    ; do kernel call