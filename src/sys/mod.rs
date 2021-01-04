use crate::utils::Endpoint;
use crate::utils::MinixProcessTable;

pub fn do_kernel_call(
    _caller_endpoint: Endpoint,
    _process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    todo!();
}
