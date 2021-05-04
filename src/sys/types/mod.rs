use super::NR_SYS_CALLS;
use crate::utils::Endpoint;

mod boot_image;
mod kinfo;

pub use boot_image::*;
pub use kinfo::*;
pub use r#priv::*;

pub const PROC_NAME_LEN: usize = 16;
pub const NR_BOOT_PROCS: usize = 17;
pub const NR_SYS_PROCS: usize = 64;

pub const MULTIBOOT_MAX_MODS: usize = 20;
pub const MAXMEMMAP: usize = 40;
pub const MULTIBOOT_PARAM_BUF_SIZE: usize = 1024;

pub const IPCF_MAX_ELEMENTS: usize = NR_SYS_PROCS * 2;
const NR_IO_RANGE: usize = 64;
const NR_MEM_RANGE: usize = 20;
const NR_IRQ: usize = 16;

type VirBytes = u32;

#[repr(C)]
pub struct SigSet(u32, u32, u32, u32);

// r# escapes reserved names
mod r#priv {
    use std::mem::size_of;

    use super::*;
    use crate::utils::{as_buf_u8, Endpoint};

    #[repr(C)]
    pub struct Priv {
        pub s_proc_nr: i32,    /* number of associated process */
        pub s_id: i16,         /* index of this system structure */
        pub s_flags: i16,      /* PREEMTIBLE, BILLABLE, etc. */
        pub s_init_flags: i32, /* initialization flags given to the process */

        // Asynchronous sends
        pub s_asyntab: VirBytes,      /* addr. of table in process's address space */
        pub s_asynsize: u32,          /* number of elements in table. 0 when not in use */
        pub s_asynendpoint: Endpoint, /* the endpoint the asyn table belongs to */

        pub s_trap_mask: i16, /* allowed system call traps */
        pub s_ipc_to: SysMap, /* allowed destination processes */

        // allowed kernel calls
        pub s_k_call_mask: [BitChunk; SYS_CALL_MASK_SIZE],

        pub s_sig_mgr: Endpoint,      /* signal manager for system signals */
        pub s_bak_sig_mgr: Endpoint,  /* backup signal manager for system signals */
        pub s_notify_pending: SysMap, /* bit map with pending notifications */
        pub s_asyn_pending: SysMap,   /* bit map with pending asyn messages */
        pub s_int_pending: u32,       /* pending hardware interrupts */
        pub s_sig_pending: SigSet,    /* pending signals */
        // type is pointer to IpcFilter
        pub s_ipcf: u32,              /* ipc filter (NULL when no filter is set) */

        pub s_alarm_timer: MinixTimer, /* synchronous alarm timer */
        pub s_stack_guard: u32,        /* stack guard word for kernel tasks */

        pub s_diag_sig: u8, /* send a SIGKMESS when diagnostics arrive? */

        pub s_nr_io_range: i32, /* allowed I/O ports */
        pub s_io_tab: [IoRange; NR_IO_RANGE],

        pub s_nr_mem_range: i32, /* allowed memory ranges */
        pub s_mem_tab: [MinixMemRange; NR_MEM_RANGE],

        pub s_nr_irq: i32, /* allowed IRQ lines */
        pub s_irq_tab: [i32; NR_IRQ],
        pub s_grant_table: VirBytes,    /* grant table address of process, or 0 */
        pub s_grant_entries: i32,       /* no. of entries, or 0 */
        pub s_grant_endpoint: Endpoint, /* the endpoint the grant table belongs to */
        pub s_state_table: VirBytes,    /* state table address of process, or 0 */
        pub s_state_entries: i32,       /* no. of entries, or 0 */
    }

    impl Default for Priv {
        fn default() -> Self {
            unsafe { std::mem::transmute([0u8; std::mem::size_of::<Self>()]) }
        }
    }

    impl Priv {
        pub fn as_buf(&self) -> [u8; size_of::<Priv>()] {
            as_buf_u8(self)
        }
    }
}

#[repr(C)]
pub struct SysMap {
    chunk: [BitChunk; bitmap_chunks(NR_SYS_PROCS)],
}

const BITCHUNK_BITS: usize = std::mem::size_of::<BitChunk>() * 8;
const fn bitmap_chunks(nr_bits: usize) -> usize {
    (nr_bits + BITCHUNK_BITS - 1) / BITCHUNK_BITS
}
const SYS_CALL_MASK_SIZE: usize = bitmap_chunks(NR_SYS_CALLS);

#[repr(C)]
pub struct BitChunk(u32);

#[repr(C)]
struct IpcFilter {
    r#type: i32,
    num_elements: i32,
    flags: i32,
    next: u32,
    elements: [IpcFilterElement; IPCF_MAX_ELEMENTS],
}

#[repr(C)]
struct IpcFilterElement {
    flags: i32,
    m_source: Endpoint,
    m_type: i32,
}

#[repr(C)]
pub struct MinixTimer {
    tmr_next: u32,     // next in a timer chain, type is MinixTimer*
    tmr_exp_time: u32, // expiration time (type is unsigned int or long)
    tmr_func: u32,     // function to call when expired
    tmr_arg: i32,      // integer argument
}

#[repr(C)]
pub struct IoRange {
    ior_base: u32,
    ior_limit: u32,
}

#[repr(C)]
pub struct MinixMemRange {
    mr_base: u32,
    mr_limit: u32,
}
