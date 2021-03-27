pub use boot_image::*;

mod boot_image {
    use std::mem::size_of;

    use crate::utils::endpoint::*;

    #[repr(C)]
    pub struct BootImage {
        pub proc_nr: i32,
        pub proc_name: ProcName,
        pub endpoint: Endpoint,
        pub start_addr: u32,
        pub len: u32,
    }

    impl BootImage {
        fn new(proc_nr: i32, proc_name: impl Into<ProcName>) -> Self {
            Self {
                proc_nr,
                proc_name: proc_name.into(),
                endpoint: proc_nr, // TODO: fix this, this is cheating
                start_addr: 0,
                len: 0,
            }
        }

        // TODO: change this into a static constant
        pub fn image() -> [u64; NR_BOOT_PROCS * size_of::<BootImage>() / 8] {
            static_assertions::const_assert_eq!(0, NR_BOOT_PROCS * size_of::<BootImage>() % 8);

            let image: [BootImage; NR_BOOT_PROCS] = [
                Self::new(ASYNCM, b"asyncm"),
                Self::new(IDLE, b"idle"),
                Self::new(CLOCK, b"clock"),
                Self::new(SYSTEM, b"system"),
                Self::new(HARDWARE, b"kernel"),
                Self::new(DS_PROC_NR, b"ds"),
                Self::new(RS_PROC_NR, b"rs"),
                Self::new(PM_PROC_NR, b"pm"),
                Self::new(SCHED_PROC_NR, b"sched"),
                Self::new(VFS_PROC_NR, b"vfs"),
                Self::new(MEM_PROC_NR, b"memory"),
                Self::new(TTY_PROC_NR, b"tty"),
                Self::new(MIB_PROC_NR, b"mib"),
                Self::new(VM_PROC_NR, b"vm"),
                Self::new(PFS_PROC_NR, b"pfs"),
                Self::new(MFS_PROC_NR, b"mfs"),
                Self::new(INIT_PROC_NR, b"init"),
            ];

            unsafe { std::mem::transmute(image) }
        }
    }

    pub const PROC_NAME_LEN: usize = 16;
    pub const NR_BOOT_PROCS: usize = 17;

    #[repr(C)]
    pub struct ProcName {
        bytes: [u8; PROC_NAME_LEN],
    }

    impl<const N: usize> From<&[u8; N]> for ProcName {
        fn from(name: &[u8; N]) -> Self {
            let mut bytes = [0; PROC_NAME_LEN];
            let len = usize::min(PROC_NAME_LEN, name.len());

            bytes[0..len].clone_from_slice(&name[0..len]);

            Self { bytes }
        }
    }
}

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

type VirBytes = u32;

pub const MULTIBOOT_MAX_MODS: usize = 20;
pub const MAXMEMMAP: usize = 40;
pub const MULTIBOOT_PARAM_BUF_SIZE: usize = 1024;
