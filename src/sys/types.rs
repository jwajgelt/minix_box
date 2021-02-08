use crate::utils::Endpoint;

#[repr(C)]
pub struct boot_image {
    pub proc_nr: i32,
    pub proc_name: [u8; PROC_NAME_LEN],
    pub endpoint: Endpoint,
    pub start_addr: u32,
    pub len: u32,
}

pub const PROC_NAME_LEN: usize = 16;

pub const IMAGE: [boot_image; NR_BOOT_PROCS] = [];
pub const NR_BOOT_PROCS: usize = 0;
