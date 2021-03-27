use std::mem::size_of;

use super::*;
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
