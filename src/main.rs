use nix::sys::wait::wait;
use nix::sys::{signal::Signal::SIGSEGV, wait::WaitStatus};

use utils::{priv_flags, MinixProcessTable, SharedImage, SharedMemory};
use utils::{Instruction, MinixProcess};

const HZ: u32 = 16 * 1024 * 1024; // arbitrary 16 MHz

#[macro_use]
extern crate static_assertions;

mod ipc;
mod sys;
mod utils;

fn main() {
    let mut process_table = MinixProcessTable::new();

    // prepare the usermapped memory
    let usermapped_mem = SharedMemory::new("minix_usermapped", 4096).unwrap();
    let usermapped = SharedImage::default();
    usermapped_mem.write(0, &usermapped).unwrap();

    // setup the boot processes
    let mut rs = MinixProcess::spawn("server_bin/rs").unwrap();
    rs.s_flags = priv_flags::ROOT_SYS_PROC;

    let _ = process_table.insert(rs, utils::endpoint::RS_PROC_NR);
    let _ = process_table.insert(MinixProcess::spawn("server_bin/is").unwrap(), 12); // arbitrary endpoint for testing
    let _ = process_table.insert(MinixProcess::spawn("server_bin/ipc").unwrap(), 13); // arbitrary endpoint for testing

    main_loop(&mut process_table).unwrap();
}

fn main_loop(process_table: &mut MinixProcessTable) -> Result<(), nix::Error> {
    loop {
        match wait().unwrap() {
            WaitStatus::Stopped(pid, SIGSEGV) => {
                // on SIGSEGV, check if segfault was caused by INT 0x20 or INT 0x21
                // if yes, we've got a kernel call / ipc call
                // else, cause minix SIGSEGV in process
                let caller_endpoint = process_table.pid_to_endpoint(pid).unwrap(); // these unwraps should - in general - be safe, since the only children should be minix processes
                let instruction = process_table
                    .get(caller_endpoint)
                    .unwrap()
                    .read_instruction()
                    .unwrap();

                match instruction {
                    Instruction::Int(0x20) => {
                        // kernel call
                        // TODO: optional logging
                        sys::do_kernel_call(caller_endpoint, process_table).unwrap();
                    }
                    Instruction::Int(0x21) => {
                        // ipc call
                        // TODO: optional logging
                        ipc::do_ipc(caller_endpoint, process_table).unwrap();
                    }
                    _ => {
                        // other
                        let _ = process_table
                            .get(caller_endpoint)
                            .unwrap()
                            .cause_signal(SIGSEGV);
                    }
                }
            }
            WaitStatus::Stopped(pid, sig) => {
                // received other signal than SIGSEGV
                // TODO: cause a minix signal in process
                // think about how to resume the process
                // (probably will clear up after investigating
                // how Minix handles signals)
                let _ = process_table.get_by_pid(pid).unwrap().cause_signal(sig);
            }
            WaitStatus::Exited(_, _) => todo!("process exited normally"),
            WaitStatus::Signaled(_, _, _) => panic!("process was killed by a (linux) signal. Problematic?"),
            WaitStatus::PtraceEvent(_, _, _) => unreachable!("probably unused and will be ignored"),
            WaitStatus::PtraceSyscall(_) => todo!("processes shouldn't call syscalls, so this should be ignored. Or kill process as misbehaving?"),
            WaitStatus::Continued(_) => unreachable!("WCONTINUED was not set, so this won't happen"),
            WaitStatus::StillAlive => unreachable!("WNOHANG was not set, so this won't happen"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "not yet implemented: Handle invalid endpoints. Endpoint: 0")]
    fn send_receive_test() {
        let mut process_table = MinixProcessTable::new();

        let _ = process_table.insert(MinixProcess::spawn("test_bin/sender_main").unwrap(), 41);
        let _ = process_table.insert(MinixProcess::spawn("test_bin/receiver").unwrap(), 42);

        main_loop(&mut process_table).unwrap();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Handle invalid endpoints. Endpoint: 0")]
    fn sendrec_test() {
        let mut process_table = MinixProcessTable::new();

        let _ = process_table.insert(MinixProcess::spawn("test_bin/sendrec_39").unwrap(), 39);
        let _ = process_table.insert(MinixProcess::spawn("test_bin/sendrec_40").unwrap(), 40);

        main_loop(&mut process_table).unwrap();
    }
}
