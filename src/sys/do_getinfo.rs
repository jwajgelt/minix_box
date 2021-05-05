use crate::utils::{
    minix_errno::*, Endpoint, Message, MessagePayload, MinixProcess, MinixProcessTable, Payload,
};

pub fn do_getinfo(
    caller: Endpoint,
    message: Message,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    let message: MessageSysGetInfo = Payload::from_payload(&message.payload);

    match message.request {
        request::GET_IMAGE => {
            let data = crate::sys::types::BootImage::image();
            write_result(&data, message, &mut process_table[caller])
        }
        request::GET_PRIV => {
            let caller = &mut process_table[caller];
            let data = caller.privileges.as_buf();
            write_result(&data, message, caller)
        }
        request::GET_MONPARAMS => {
            // TODO: check what the params actually look like and implement it correctly here
            let data = [0u8; super::MULTIBOOT_PARAM_BUF_SIZE];
            write_result(&data, message, &mut process_table[caller])
        }
        request::GET_WHOAMI => get_whoami(caller, &mut process_table[caller]),
        request::GET_HZ => {
            // TODO: think of a good HZ value to report
            if message.val_len > 0 && (message.val_len as usize) < 4 {
                return Ok(E2BIG);
            }
            process_table
                .get_mut(caller)
                .unwrap()
                .write_32(message.val_ptr as u64, crate::HZ)?;
            Ok(OK)
        }
        request => {
            panic!("do_getinfo: invalid request {}", request);
            // Ok(EINVAL) // TODO: return EINVAL instead
        }
    }
}

fn write_result(
    data: &[u8],
    message: MessageSysGetInfo,
    caller: &mut MinixProcess,
) -> Result<i32, nix::Error> {
    if message.val_len > 0 && (message.val_len as usize) < data.len() / 8 {
        return Ok(E2BIG);
    }

    // TODO: get around the fact that we have to write 64-bit segments
    caller.write_buf_u8(message.val_ptr as u64, data)?;

    Ok(OK)
}

fn get_whoami(caller_endpoint: Endpoint, caller: &mut MinixProcess) -> Result<i32, nix::Error> {
    let response = MessageSysWhoAmI {
        endpt: caller_endpoint,
        privflags: caller.s_flags as i32,
        initflags: 0,
        name: [0; 44],
    };

    // write the response to the original message,
    // pointed to by the rax register
    // (+8, since we skip the source and type fields, and only write the payload)
    let regs = caller.get_regs()?;
    let data: [u32; 14] = response.into_payload();
    let data_u64: [u64; 7] = unsafe { std::mem::transmute(data) };
    caller.write_buf(regs.rax + 8, &data_u64)?;

    Ok(OK)
}

/// the getinfo() kernel call request message
#[repr(C)]
#[derive(Debug)]
struct MessageSysGetInfo {
    request: i32,
    endpt: Endpoint,
    val_ptr: u32,
    val_len: i32,
    val_ptr2: u32,
    val_len2_e: i32,

    padding: [u8; 32],
}
assert_eq_size!(MessageSysGetInfo, MessagePayload);
impl Payload for MessageSysGetInfo {}

/// response to the getinfo() WHOAMI call
#[repr(C)]
#[derive(Debug)]
struct MessageSysWhoAmI {
    endpt: Endpoint,
    privflags: i32,
    initflags: i32,
    name: [u8; 44],
}
assert_eq_size!(MessageSysWhoAmI, MessagePayload);
impl Payload for MessageSysWhoAmI {}

#[allow(dead_code)]
mod request {
    pub const GET_KINFO: i32 = 0; /* get kernel information structure */
    pub const GET_IMAGE: i32 = 1; /* get system image table */
    pub const GET_PROCTAB: i32 = 2; /* get kernel process table */
    pub const GET_RANDOMNESS: i32 = 3; /* get randomness buffer */
    pub const GET_MONPARAMS: i32 = 4; /* get monitor parameters */
    pub const GET_KENV: i32 = 5; /* get kernel environment string */
    pub const GET_IRQHOOKS: i32 = 6; /* get the IRQ table */
    pub const GET_PRIVTAB: i32 = 8; /* get kernel privileges table */
    pub const GET_KADDRESSES: i32 = 9; /* get various kernel addresses */
    pub const GET_SCHEDINFO: i32 = 10; /* get scheduling queues */
    pub const GET_PROC: i32 = 11; /* get process slot if given process */
    pub const GET_MACHINE: i32 = 12; /* get machine information */
    pub const GET_LOCKTIMING: i32 = 13; /* get lock()/unlock() latency timing */
    pub const GET_BIOSBUFFER: i32 = 14; /* get a buffer for BIOS calls */
    pub const GET_LOADINFO: i32 = 15; /* get load average information */
    pub const GET_IRQACTIDS: i32 = 16; /* get the IRQ masks */
    pub const GET_PRIV: i32 = 17; /* get privilege structure */
    pub const GET_HZ: i32 = 18; /* get HZ value */
    pub const GET_WHOAMI: i32 = 19; /* get own name, endpoint, and privileges */
    pub const GET_RANDOMNESS_BIN: i32 = 20; /* get one randomness bin */
    pub const GET_IDLETSC: i32 = 21; /* get cumulative idle time stamp counter */
    pub const GET_CPUINFO: i32 = 23; /* get information about cpus */
    pub const GET_REGS: i32 = 24; /* get general process registers */
    pub const GET_CPUTICKS: i32 = 25; /* get per-state ticks for a cpu */
}
