use crate::utils::Endpoint;
use crate::utils::MinixProcessTable;

pub fn do_kernel_call(
    caller_endpoint: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    let process = process_table.get_mut(caller_endpoint).unwrap();

    // the kernel call message address is stored
    // in the eax register
    let addr = process.get_regs()?.rax;
    let message = process.read_message(addr)?;

    // kernel call number is sent in the 
    // m_type field of the message
    let call_nr = message.m_type;

    todo!("Kernel call nr: {:#x} from {}", call_nr, caller_endpoint);
}
