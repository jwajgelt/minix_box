#include <minix/syslib.h>
#include <stdlib.h>
#include <string.h>

void _start() {
    message m;
    memset(&m, 0, sizeof(m));
    m.m_u8.data[0] = 40;
    ipc_sendrec(39, &m);
    exit(0);
}
