use std::mem::size_of;

const KERNINFO_MAGIC: u32 = 0xfc3b84bf;

/// the minix_kerninfo structure, shared by all processes
#[repr(C)]
pub struct MinixKerninfo {
    pub kerninfo_magic: u32,
    pub minix_feature_flags: u32, // features in minix kernel
    pub ki_flags: u32,            // what is present in this struct
    pub flags_unused2: u32,
    pub flags_unused3: u32,
    pub flags_unused4: u32,
    pub kinfo_ptr: u32,
    pub machine_ptr: u32,
    pub kmessages_ptr: u32,
    pub loadinfo_ptr: u32,
    pub minix_ipcvecs_ptr: u32,
    pub kuserinfo_ptr: u32,
    pub arm_frclock: u32,
    pub kclockinfo_ptr: u32,
}

impl Default for MinixKerninfo {
    fn default() -> Self {
        // init all fields with 0 for now, hope NULLs are ok here
        // TODO: fix this
        Self {
            kerninfo_magic: KERNINFO_MAGIC,
            minix_feature_flags: 0,
            ki_flags: 0,
            flags_unused2: 0,
            flags_unused3: 0,
            flags_unused4: 0,
            kinfo_ptr: 0,
            machine_ptr: 0,
            kmessages_ptr: 0,
            loadinfo_ptr: 0,
            minix_ipcvecs_ptr: 0,
            kuserinfo_ptr: 0,
            arm_frclock: 0,
            kclockinfo_ptr: 0,
        }
    }
}

// These types are defined in `include/minix/type.h`
#[repr(C)]
pub struct Clockinfo {
    pub boottime: u32, // : time_t;  number of seconds since UNIX epoch
    pub uptime: u32,   // : clock_t; number of clock ticks since system boot
    pub _rsvd1: u32,   // reserved for 64-bit uptime
    pub realtime: u32, // : clock_t; real time in clock ticks since boot
    pub _rsvd2: u32,   // reserved for 64-bit real time
    pub hz: u32,       // clock frequency in ticks per second
}

impl Default for Clockinfo {
    fn default() -> Self {
        Self {
            boottime: 0,
            uptime: 0,
            _rsvd1: 0,
            realtime: 0,
            _rsvd2: 0,
            hz: crate::HZ,
        }
    }
}

pub const SHARED_BASE_ADDR: u32 = 0xf1002000;

#[repr(C)]
pub struct SharedImage {
    pub minix_kerninfo: MinixKerninfo,
    pub kclockinfo: Clockinfo,
}

impl Default for SharedImage {
    fn default() -> Self {
        let mut image = Self {
            minix_kerninfo: MinixKerninfo::default(),
            kclockinfo: Clockinfo::default(),
        };

        image.minix_kerninfo.kclockinfo_ptr = SHARED_BASE_ADDR + size_of::<MinixKerninfo>() as u32;

        image
    }
}
