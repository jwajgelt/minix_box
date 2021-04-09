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
        s_proc_nr: i32,    /* number of associated process */
        s_id: i16,         /* index of this system structure */
        s_flags: i16,      /* PREEMTIBLE, BILLABLE, etc. */
        s_init_flags: i32, /* initialization flags given to the process */

        // Asynchronous sends
        s_asyntab: VirBytes,      /* addr. of table in process' address space */
        s_asynsize: u32,          /* number of elements in table. 0 when not in use */
        s_asynendpoint: Endpoint, /* the endpoint the asyn table belongs to */

        s_trap_mask: i16, /* allowed system call traps */
        s_ipc_to: SysMap, /* allowed destination processes */

        // allowed kernel calls
        s_k_call_mask: [BitChunk; SYS_CALL_MASK_SIZE],

        s_sig_mgr: Endpoint,      /* signal manager for system signals */
        s_bak_sig_mgr: Endpoint,  /* backup signal manager for system signals */
        s_notify_pending: SysMap, /* bit map with pending notifications */
        s_asyn_pending: SysMap,   /* bit map with pending asyn messages */
        s_int_pending: u32,       /* pending hardware interrupts */
        s_sig_pending: SigSet,    /* pending signals */
        s_ipcf: u32,              /* ipc filter (NULL when no filter is set) */

        // type is pointer to IpcFilter
        s_alarm_timer: MinixTimer, /* synchronous alarm timer */
        s_stack_guard: u32,        /* stack guard word for kernel tasks */

        s_diag_sig: u8, /* send a SIGKMESS when diagnostics arrive? */

        s_nr_io_range: i32, /* allowed I/O ports */
        s_io_tab: [IoRange; NR_IO_RANGE],

        s_nr_mem_range: i32, /* allowed memory ranges */
        s_mem_tab: [MinixMemRange; NR_MEM_RANGE],

        s_nr_irq: i32, /* allowed IRQ lines */
        s_irq_tab: [i32; NR_IRQ],
        s_grant_table: VirBytes,    /* grant table address of process, or 0 */
        s_grant_entries: i32,       /* no. of entries, or 0 */
        s_grant_endpoint: Endpoint, /* the endpoint the grant table belongs to */
        s_state_table: VirBytes,    /* state table address of process, or 0 */
        s_state_entries: i32,       /* no. of entries, or 0 */
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
struct SysMap {
    chunk: [BitChunk; bitmap_chunks(NR_SYS_PROCS)],
}

const BITCHUNK_BITS: usize = std::mem::size_of::<BitChunk>() * 8;
const fn bitmap_chunks(nr_bits: usize) -> usize {
    (nr_bits + BITCHUNK_BITS - 1) / BITCHUNK_BITS
}
const SYS_CALL_MASK_SIZE: usize = bitmap_chunks(NR_SYS_CALLS);

#[repr(C)]
struct BitChunk(u32);

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
struct MinixTimer {
    tmr_next: u32,     // next in a timer chain, type is MinixTimer*
    tmr_exp_time: u32, // expiration time (type is unsigned int or long)
    tmr_func: u32,     // function to call when expired
    tmr_arg: i32,      // integer argument
}

#[repr(C)]
struct IoRange {
    ior_base: u32,
    ior_limit: u32,
}

#[repr(C)]
struct MinixMemRange {
    mr_base: u32,
    mr_limit: u32,
}
