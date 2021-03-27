use super::*;

#[repr(C)]
#[allow(dead_code)]
pub struct KInfo {
    // Straight multiboot-provided info
    pub mbi: u32,
    pub module_list: [u32; MULTIBOOT_MAX_MODS],
    pub memmap: [u32; MAXMEMMAP],
    pub mem_high_phys: u32,
    pub mmap_size: i32,

    // Multiboot-derived
    pub mods_with_kernel: i32,
    pub kern_mod: i32,

    // Minix stuff, started at bootstrap phase
    pub freepde_start: i32, // lowest pde unused kernel pde
    pub param_buf: [u8; MULTIBOOT_PARAM_BUF_SIZE],

    // Minix stuff
    pub kmessages: u32,
    pub do_serial_debug: i32,     /* system serial output */
    pub serial_debug_baud: i32,   /* serial baud rate */
    pub minix_panicing: i32,      /* are we panicing? */
    pub user_sp: VirBytes,        /* where does kernel want stack set */
    pub user_end: VirBytes,       /* upper proc limit */
    pub vir_kern_start: VirBytes, /* kernel addrspace starts */
    pub bootstrap_start: VirBytes,
    pub bootstrap_len: VirBytes,
    pub boot_procs: [BootImage; NR_BOOT_PROCS],
    pub nr_procs: i32,                       /* number of user processes */
    pub nr_tasks: i32,                       /* number of kernel tasks */
    pub release: [u8; 6],                    /* kernel release number */
    pub version: [u8; 6],                    /* kernel version number */
    pub vm_allocated_bytes: i32,             /* allocated by kernel to load vm */
    pub kernel_allocated_bytes: i32,         /* used by kernel */
    pub kernel_allocated_bytes_dynamic: i32, /* used by kernel (runtime) */
}
