use crate::utils::{Endpoint, Message, MessagePayload, MinixProcess, MinixProcessTable, Payload};

pub fn do_getinfo(
    caller: Endpoint,
    message: Message,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    let message: MessageSysGetInfo = Payload::from_payload(&message.payload);

    println!("getinfo() request {}", message.request);

    match message.request {
        request::GET_IMAGE => {
            println!("In GET_IMAGE");
            panic!();
        }
        request::GET_WHOAMI => get_whoami(caller, process_table.get_mut(caller).unwrap()),
        request => {
            println!("do_getinfo: invalid request {}", request);
            panic!();
            // Ok(-1) // TODO: return EINVAL instead
        }
    }
}

fn get_whoami(caller_endpoint: Endpoint, caller: &mut MinixProcess) -> Result<i32, nix::Error> {
    let response = MessageSysWhoAmI {
        endpt: caller_endpoint,
        privflags: 0,
        initflags: 0,
        name: [0; 44],
    };

    // write the response to the original message,
    // pointed to by the rax register
    // (+8, since we skip the source and type fields, and only write the payload)
    let regs = caller.get_regs()?;
    caller.write_buf(regs.rax + 8, &response.into_payload())?;

    Ok(0)
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
