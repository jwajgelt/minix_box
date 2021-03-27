use crate::utils::{Endpoint, Message, MinixProcessTable};

pub fn do_exit(
    caller: Endpoint,
    _: Message,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    println!("{:X?}", process_table[caller].get_regs());
    unimplemented!();
}
