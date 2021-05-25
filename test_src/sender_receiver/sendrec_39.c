#include <minix/syslib.h>
#include <stdlib.h>

void _start() {
    int status;
    message m;
    ipc_receive(40, &m, &status);
    m.m_u8.data[1] = m.m_u8.data[0] + 1;
    ipc_send(40, &m);
    exit(0);
}
