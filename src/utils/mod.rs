mod message;
mod message_queue;
mod minix_process;
mod minix_process_table;

#[allow(dead_code)]
pub mod minix_errno;

pub use endpoint::Endpoint;
pub use message::*;
pub use minix_process::*;
pub use minix_process_table::*;

/// cast a slice of u8 to a slice of u64
/// this is dangerous if the length of the original slice
/// or the offset it begins at is not divisible by 8
// pub fn u8_to_u64(buf: &[u8]) -> &[u64] {
//     assert!(buf.len() % 8 == 0);
//     unsafe {
//         let ptr = buf.as_ptr();
//         assert!(ptr as usize % 8 == 0);
//         let result = std::slice::from_raw_parts(ptr as *const u64, buf.len() / 8);
//         result
//     }
// }

#[allow(dead_code)]
pub mod endpoint {
    // special endpoints defined in minix/endpoint.h
    pub type Endpoint = i32;
    const ENDPOINT_GENERATION_SHIFT: Endpoint = 15;
    const ENDPOINT_GENERATION_SIZE: Endpoint = 1 << ENDPOINT_GENERATION_SHIFT;
    const ENDPOINT_SLOT_TOP: Endpoint = ENDPOINT_GENERATION_SIZE - MAX_NR_TASKS; // ENDPOINT_GENERATION_SIZE - MAX_NR_TASKS

    pub const ANY: Endpoint = ENDPOINT_SLOT_TOP - 1;
    pub const NONE: Endpoint = ENDPOINT_SLOT_TOP - 2;
    pub const SELF: Endpoint = ENDPOINT_SLOT_TOP - 3;

    // special task/process numbers. These are all defined in `include/minix/com.h`
    pub const ASYNCM: Endpoint = -5;
    pub const IDLE: Endpoint = -4;
    pub const CLOCK: Endpoint = -3;
    pub const SYSTEM: Endpoint = -2;
    pub const KERNEL: Endpoint = -1;
    pub const HARDWARE: Endpoint = KERNEL;

    // Number of tasks
    pub const MAX_NR_TASKS: Endpoint = 1023;
    pub const NR_TASKS: Endpoint = 5;

    // user-space processes with special proc numbers
    pub const PM_PROC_NR: Endpoint = 0; /* process manager */
    pub const VFS_PROC_NR: Endpoint = 1; /* file system */
    pub const RS_PROC_NR: Endpoint = 2; /* reincarnation server */
    pub const MEM_PROC_NR: Endpoint = 3; /* memory driver (RAM disk, null, etc.) */
    pub const SCHED_PROC_NR: Endpoint = 4; /* scheduler */
    pub const TTY_PROC_NR: Endpoint = 5; /* terminal (TTY) driver */
    pub const DS_PROC_NR: Endpoint = 6; /* data store server */
    pub const MIB_PROC_NR: Endpoint = 7; /* management info base service */
    pub const VM_PROC_NR: Endpoint = 8; /* memory server */
    pub const PFS_PROC_NR: Endpoint = 9; /* pipe filesystem */
    pub const MFS_PROC_NR: Endpoint = 10; /* minix root filesystem */

    pub const LAST_SPECIAL_PROC_NR: Endpoint = 11;
    pub const INIT_PROC_NR: Endpoint = LAST_SPECIAL_PROC_NR;
    pub const NR_BOOT_MODULES: Endpoint = INIT_PROC_NR + 1;
}

pub fn as_buf_u8<T, const N: usize>(val: &T) -> [u8; N] {
    use std::mem::size_of;
    assert_eq!(N, size_of::<T>());
    unsafe { std::mem::transmute_copy(val) }
}
