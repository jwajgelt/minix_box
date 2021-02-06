use std::mem::size_of;

use super::Endpoint;

/// this struct represents a Minix ipc message
#[repr(C)]
#[derive(Debug)]
pub struct Message {
    pub source: Endpoint,   // the endpoint sending the message
    pub m_type: u32,        // type of the message
    pub payload: [u64; 7],  // the payload of the message
}

/// the size of the Message struct, in bytes,
/// needs to be equal to this number
pub const MESSAGE_SIZE: usize = 64;
const_assert_eq!(MESSAGE_SIZE, size_of::<Message>());
assert_eq_size!([u64; MESSAGE_SIZE / 8], Message);

// we implement From/Into [u64] for easier conversions
// when reading / writing into traced process memory
impl From<[u64; MESSAGE_SIZE / 8]> for Message {
    fn from(buf: [u64; 8]) -> Self {
        unsafe { std::mem::transmute(buf) }
    }
}

impl Into<[u64; MESSAGE_SIZE / 8]> for Message {
    fn into(self) -> [u64; 8] {
        unsafe { std::mem::transmute(self) }
    }
}