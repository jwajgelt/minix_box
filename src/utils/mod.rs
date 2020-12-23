mod minix_process;
mod minix_process_table;

pub const MESSAGE_SIZE: usize = 64;

pub use minix_process::MinixProcess;
pub use minix_process::ProcessState;
pub use minix_process_table::Endpoint;
pub use minix_process_table::MinixProcessTable;
