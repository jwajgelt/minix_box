use crate::utils::{
    minix_errno::{EINVAL, OK},
    Endpoint, Message, MessagePayload, MinixProcessTable, Payload,
};

pub fn do_diagctl(
    caller: Endpoint,
    message: Message,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // I do think we care about this call
    println!("In diagctl");

    let message: MessageSysDiagCtl = Payload::from_payload(&message.payload);

    println!("sys_diagctl call {:#x}", message.code);

    match message.code {
        CODE_DIAG => {
            let caller = &mut process_table[caller];

            let mybuf = caller.read_buf_u8(message.buf as u64, message.len as usize)?;

            if let Ok(message) = std::str::from_utf8(&mybuf) {
                println!("diagctl: {:?}", message);
            } else {
                println!("diagctl couldn't convert message to string");
            }

            Ok(OK)
        }
        CODE_STACKTRACE | CODE_REGISTER | CODE_UNREGISTER => {
            unimplemented!("do_diagctl: unimplemented request {}", message.code);
        }
        _ => {
            println!("do_diagctl: invalid request {}", message.code);
            Ok(EINVAL)
        }
    }
}

/// the sys_diagctl() kernel call request message
#[repr(C)]
#[derive(Debug)]
struct MessageSysDiagCtl {
    code: i32,
    buf: u32,
    len: i32,
    endpoint: Endpoint,

    padding: [u8; 40],
}
assert_eq_size!(MessageSysDiagCtl, MessagePayload);
impl Payload for MessageSysDiagCtl {}

const CODE_DIAG: i32 = 1;
const CODE_STACKTRACE: i32 = 2;
const CODE_REGISTER: i32 = 3;
const CODE_UNREGISTER: i32 = 4;
