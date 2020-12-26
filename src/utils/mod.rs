mod minix_process;
mod minix_process_table;

pub const MESSAGE_SIZE: usize = 64;

pub use minix_process::*;
pub use minix_process_table::*;
