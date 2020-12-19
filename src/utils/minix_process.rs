use super::MESSAGE_SIZE;
use nix::libc::user_regs_struct;
use nix::sys::ptrace;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::execv;
use nix::unistd::fork;
use nix::unistd::Pid;
use std::{
    ffi::{c_void, CString},
    mem::size_of_val,
};

pub struct MinixProcess {
    pub pid: Pid,
}

impl MinixProcess {
    /// spawns a child process running a given program,
    /// returning a MinixProcess struct representing that
    /// process
    pub fn spawn(path: &str) -> Result<Self, nix::Error> {
        use nix::unistd::ForkResult::*;
        match unsafe { fork() } {
            Ok(Parent { child }) => {
                if let WaitStatus::Exited(_, _) = waitpid(child, None)? {
                    return Err(nix::Error::Sys(nix::errno::Errno::ECHILD)); // error doesn't make sense, should think error handling through
                };

                let minix_process = Self { pid: child };

                ptrace::cont(child, None)?;
                Ok(minix_process)
            }
            Ok(Child) => {
                ptrace::traceme().unwrap();
                execv::<&CString>(&CString::new(path).unwrap(), &[]).unwrap();
                unreachable!()
            }
            Err(e) => Err(e),
        }
    }

    /// returns the values of registers in the traced process
    pub fn get_regs(self: &Self) -> Result<user_regs_struct, nix::Error> {
        ptrace::getregs(self.pid)
    }

    /// reads one word (8 bytes) from an address
    /// in the traced process's memory
    pub fn read(self: &Self, addr: u64) -> Result<i64, nix::Error> {
        let addr: *mut c_void = unsafe { std::mem::transmute(addr) };
        ptrace::read(self.pid, addr)
    }

    /// reads a message the process is sending:
    /// reads 64 bytes from memory pointed to
    /// by the eax register
    pub fn retrieve_message(self: &Self) -> Result<[u8; MESSAGE_SIZE], nix::Error> {
        let mut result = [0; MESSAGE_SIZE];
        let result_i64: &mut [i64; MESSAGE_SIZE / 8] =
            unsafe { &mut *(result.as_mut_ptr() as *mut [i64; MESSAGE_SIZE / 8]) };
        assert_eq!(size_of_val(&result), size_of_val(result_i64));

        let addr = self.get_regs()?.rax;
        for (i, data) in result_i64.iter_mut().enumerate() {
            *data = self.read(addr + i as u64)?
        }

        Ok(result)
    }
}

// since a MinixProcess value corresponds to a running process,
// we should probably terminate the process when the value is dropped.
impl Drop for MinixProcess {
    fn drop(self: &mut Self) {
        // some thought should probably be given here,
        // let's just kill the process for now.
        // errors here can probably be safely ignored?
        let _ = kill(self.pid, Some(Signal::SIGKILL));
    }
}
