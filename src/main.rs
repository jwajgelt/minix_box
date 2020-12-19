use nix::sys::ptrace;
use nix::sys::wait::wait;
use nix::sys::{signal::Signal::SIGSEGV, wait::WaitStatus};

use utils::MinixProcess;

mod utils;

fn main() {
    let sender = MinixProcess::spawn("sender").unwrap();

    let receiver = MinixProcess::spawn("receiver").unwrap();

    loop {
        match wait().unwrap() {
            WaitStatus::Stopped(pid, SIGSEGV) => {
                // TODO: on SIGSEGV, check if segfault was caused by INT 0x20 or INT 0x21
                // if yes, we've got a kernel call / ipc call
                // else, cause minix SIGSEGV in process
                if pid == sender.pid {
                    let regs = sender.get_regs();
                    println!("got call from sender {:#x?}", regs);
                    if let Ok(message) = receiver.retrieve_message() {
                        println!("message: {:#04x?}", &message);
                    }
                } else if pid == receiver.pid {
                    let regs = receiver.get_regs();
                    println!("got call from receiver {:#x?}", regs);
                    if let Ok(message) = receiver.retrieve_message() {
                        println!("message: {:#04x?}", &message);
                    }
                }
            }
            WaitStatus::Stopped(pid, sig) => {
                // received other signal than SIGSEGV
                // TODO: cause a minix signal in process
                // think about how to resume the process
                // (probably will clear up after investigating
                // how Minix handles signals)
                let _ = ptrace::cont(pid, sig);
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
