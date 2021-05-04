use std::mem::size_of;

use super::Endpoint;

/// this struct represents a Minix ipc message
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub source: Endpoint,        // the endpoint sending the message
    pub m_type: u32,             // type of the message
    pub payload: MessagePayload, // the payload of the message
}

pub type MessagePayload = [u32; 14];

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

impl From<Message> for [u64; MESSAGE_SIZE / 8] {
    fn from(msg: Message) -> Self {
        unsafe { std::mem::transmute(msg) }
    }
}

/// this trait implements the conversions
/// for message payload types.
/// When implementing this trait, one should always check
/// that size_of(Self) == size_of(MessagePayload)
pub trait Payload: Sized {
    fn from_payload(payload: &MessagePayload) -> Self {
        unsafe { std::mem::transmute_copy(payload) }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_payload(&self) -> MessagePayload {
        unsafe { std::mem::transmute_copy(self) }
    }
}

pub const NOTIFY_MESSAGE: u32 = 0x1000;
