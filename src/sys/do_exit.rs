use crate::utils::{Endpoint, Message, MinixProcessTable};

pub fn do_exit(
    _caller: Endpoint,
    _: Message,
    _process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    unimplemented!();
}
