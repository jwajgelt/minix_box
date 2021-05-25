global _start
_start:
int 3       ; trap to kernel, to be caught in test
mov ebx, [0xf1002000]   ; the address we mapped memory to in the test
mov eax, 1  ; system call number (sys_exit)
int 0x80    ; do system call