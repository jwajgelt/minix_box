use nix::sys::wait::wait;
use nix::sys::{signal::Signal::SIGSEGV, wait::WaitStatus};

use utils::MinixProcessTable;
use utils::{Instruction, MinixProcess};

mod ipc;
mod sys;
mod utils;

fn main() {
    let mut process_table = MinixProcessTable::new();

    // TODO: move this to test files
    let _ = process_table.insert(MinixProcess::spawn("sendrec_39").unwrap(), 39);
    let _ = process_table.insert(MinixProcess::spawn("sendrec_40").unwrap(), 40);
    let _ = process_table.insert(MinixProcess::spawn("sender").unwrap(), 41);
    let _ = process_table.insert(MinixProcess::spawn("receiver").unwrap(), 42);

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
                        println!("Pid {} requests kernel call", pid);
                        let _ = sys::do_kernel_call(caller_endpoint, &mut process_table);
                    }
                    Instruction::Int(0x21) => {
                        // ipc call
                        println!("Pid {}, endpoint {} requests ipc", pid, caller_endpoint);
                        let _ = ipc::do_ipc(caller_endpoint, &mut process_table);
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
