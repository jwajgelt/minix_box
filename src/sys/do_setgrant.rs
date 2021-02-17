use crate::utils::{Endpoint, Message, MinixProcessTable};

pub fn do_setgrant(
    _caller: Endpoint,
    _message: Message,
    _process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // TODO: this
    Ok(0)
}
