#include <minix/syslib.h>
#include <stdlib.h>

void _start() {
    int status;
    message m;
    ipc_receive(41, &m, &status);
    exit(0);
}
