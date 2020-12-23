use nix::sys::wait::wait;
use nix::sys::{signal::Signal::SIGSEGV, wait::WaitStatus};

use utils::MinixProcess;
use utils::MinixProcessTable;

mod ipc;
mod utils;

fn main() {
    let mut processes = MinixProcessTable::new();

    let sender = MinixProcess::spawn("sender").unwrap();

    let receiver = MinixProcess::spawn("receiver").unwrap();

    let _ = processes.insert(sender, 41);
    let _ = processes.insert(receiver, 42);

    loop {
        match wait().unwrap() {
            WaitStatus::Stopped(pid, SIGSEGV) => {
                // TODO: on SIGSEGV, check if segfault was caused by INT 0x20 or INT 0x21
                // if yes, we've got a kernel call / ipc call
                // else, cause minix SIGSEGV in process
                let caller_endpoint = processes.pid_to_endpoint(pid).unwrap(); // these unwraps should - in general - be safe, since the only children should be minix processes
                println!(
                    "rip: {:#010x?}",
                    processes
                        .get(caller_endpoint)
                        .unwrap()
                        .get_regs()
                        .unwrap()
                        .rip
                );
                let _ = ipc::handle_ipc(caller_endpoint, &mut processes);
            }
            WaitStatus::Stopped(pid, sig) => {
                // received other signal than SIGSEGV
                // TODO: cause a minix signal in process
                // think about how to resume the process
                // (probably will clear up after investigating
                // how Minix handles signals)
                let _ = processes.get_by_pid(pid).unwrap().cause_signal(sig);
            }
            WaitStatus::Exited(_, _) => {} // process exited normally. TODO: what to do?
            WaitStatus::Signaled(_, _, _) => {} // process was killed by a (linux) signal. Problematic?
            WaitStatus::PtraceEvent(_, _, _) => {} // probably unused and will be ignored
            WaitStatus::PtraceSyscall(_) => {} // processes shouldn't call syscalls, so this should be ignored. Or kill process as misbehaving?
            WaitStatus::Continued(_) => {} // WCONTINUED was not set, so this won't happen - ignore
            WaitStatus::StillAlive => {}   // WNOHANG was not set, so this won't happen - ignore
        }
    }
}
