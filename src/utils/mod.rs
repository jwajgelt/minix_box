mod message;
mod message_queue;
mod minix_process;
mod minix_process_table;

pub use message::*;
pub use minix_process::*;
pub use minix_process_table::*;
pub use endpoint::Endpoint;

#[allow(dead_code)]
pub mod endpoint {
    // special endpoints defined in minix/endpoint.h
    pub type Endpoint = i32;
    const ENDPOINT_GENERATION_SHIFT: Endpoint = 15;
    const ENDPOINT_GENERATION_SIZE: Endpoint = 1 << ENDPOINT_GENERATION_SHIFT;
    const ENDPOINT_SLOT_TOP: Endpoint = ENDPOINT_GENERATION_SIZE - 1023; // ENDPOINT_GENERATION_SIZE - MAX_NR_TASKS

    pub const ANY: Endpoint = ENDPOINT_SLOT_TOP - 1;
    pub const NONE: Endpoint = ENDPOINT_SLOT_TOP - 2;
    pub const SELF: Endpoint = ENDPOINT_SLOT_TOP - 3;
}