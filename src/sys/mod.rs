mod do_getinfo;

use crate::utils::MinixProcessTable;
use crate::utils::{Endpoint, Message};

pub fn do_kernel_call(
    caller_endpoint: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    let process = process_table.get_mut(caller_endpoint).unwrap();
    let mut regs = process.get_regs()?;

    // the kernel call message address is stored
    // in the eax register
    let addr = regs.rax;
    let mut message = process.read_message(addr).unwrap();
    message.source = caller_endpoint;

    // kernel call number is sent in the
    // m_type field of the message
    let call_nr = message.m_type as usize;
    let result;

    println!("Kernel call nr: {:#x} from {}", call_nr, caller_endpoint);

    if (KERNEL_CALL..(KERNEL_CALL + NR_KERNEL_CALLS)).contains(&call_nr) {
        result = CALL_VEC[call_nr - KERNEL_CALL](caller_endpoint, message)?;
    } else {
        unimplemented!()
    }

    regs.rcx = result as u64; // TODO: is this wrong? - have to check where the return value is stored
    process.set_regs(regs)?;
    process.cont().unwrap();

    Ok(())
}

// the kernel call numbers are defined in `include/minix/com.h`
const KERNEL_CALL: usize = 0x600;
const NR_KERNEL_CALLS: usize = 58;

type KernelCall = &'static dyn Fn(Endpoint, Message) -> Result<i32, nix::Error>;

const CALL_VEC: [KernelCall; NR_KERNEL_CALLS] = [
    &sys_unimplemented,      // 0
    &sys_unimplemented,      // 1
    &sys_unimplemented,      // 2
    &sys_unimplemented,      // 3
    &sys_unimplemented,      // 4
    &sys_unimplemented,      // 5
    &sys_unimplemented,      // 6
    &sys_unimplemented,      // 7
    &sys_unimplemented,      // 8
    &sys_unimplemented,      // 9
    &sys_unimplemented,      // 10
    &sys_unimplemented,      // 11
    &sys_unimplemented,      // 12
    &sys_unimplemented,      // 13
    &sys_unimplemented,      // 14
    &sys_unimplemented,      // 15
    &sys_unimplemented,      // 16
    &sys_unimplemented,      // 17
    &sys_unimplemented,      // 18
    &sys_unimplemented,      // 19
    &sys_unimplemented,      // 20
    &sys_unimplemented,      // 21
    &sys_unimplemented,      // 22
    &sys_unimplemented,      // 23
    &sys_unimplemented,      // 24
    &sys_unimplemented,      // 25
    &do_getinfo::do_getinfo, // 26 SYS_GETINFO
    &sys_unimplemented,      // 27
    &sys_unimplemented,      // 28
    &sys_unimplemented,      // 29
    &sys_unimplemented,      // 30
    &sys_unimplemented,      // 31
    &sys_unimplemented,      // 32
    &sys_unimplemented,      // 33
    &sys_unimplemented,      // 34
    &sys_unimplemented,      // 35
    &sys_unimplemented,      // 36
    &sys_unimplemented,      // 37
    &sys_unimplemented,      // 38
    &sys_unimplemented,      // 39
    &sys_unimplemented,      // 40
    &sys_unimplemented,      // 41
    &sys_unimplemented,      // 42
    &sys_unimplemented,      // 43
    &sys_unimplemented,      // 44
    &sys_unimplemented,      // 45
    &sys_unimplemented,      // 46
    &sys_unimplemented,      // 47
    &sys_unimplemented,      // 48
    &sys_unimplemented,      // 49
    &sys_unimplemented,      // 50
    &sys_unimplemented,      // 51
    &sys_unimplemented,      // 52
    &sys_unimplemented,      // 53
    &sys_unimplemented,      // 54
    &sys_unimplemented,      // 55
    &sys_unimplemented,      // 56
    &sys_unimplemented,      // 57
];

fn sys_unimplemented(_: Endpoint, _: Message) -> Result<i32, nix::Error> {
    unimplemented!();
}
