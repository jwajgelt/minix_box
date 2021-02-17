use crate::utils::{Endpoint, Message, MinixProcessTable};

pub fn do_statectl(
    _caller: Endpoint,
    _message: Message,
    _process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // TODO: this call
    Ok(0)
}
