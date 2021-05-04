use crate::sys::Priv;

use super::{message_queue::MessageQueue, Endpoint, Message, SharedMemory, MESSAGE_SIZE};
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
    ptr::slice_from_raw_parts,
};

#[derive(Clone, Copy)]
pub enum ProcessState {
    Running,
    Sending(Endpoint),
    Receiving(Endpoint),
    SendReceiving(Endpoint),
}

pub enum Instruction {
    Int(u8), // programmable interrupt
    Other,
}

/// a struct representing a single, running Minix process
pub struct MinixProcess {
    pid: Pid,
    pub state: ProcessState,
    pub queue: MessageQueue,
    pub reply_pending: bool,
    pub name: String,
    pub s_flags: u16,
    pub privileges: Priv,
    pub notify_pending: Vec<Endpoint>,
    pub async_pending: Vec<Endpoint>,
    pub minix_kerninfo_addr: Option<u32>,
}

impl MinixProcess {
    /// spawns a child process running a given program,
    /// returning a MinixProcess struct representing that
    /// process
    pub fn spawn(path: &str) -> Result<Self, nix::Error> {
        use nix::unistd::ForkResult::*;
        match unsafe { fork() } {
            Ok(Parent { child }) => {
                let status = waitpid(child, None)?;
                if let WaitStatus::Exited(_, _) = status {
                    return Err(nix::Error::Sys(nix::errno::Errno::ECHILD)); // error doesn't make sense, should think error handling through
                };
                if let WaitStatus::Signaled(_, _, _) = status {
                    return Err(nix::Error::Sys(nix::errno::Errno::ECHILD));
                }

                let minix_process = Self {
                    pid: child,
                    state: ProcessState::Running,
                    queue: MessageQueue::new(),
                    reply_pending: false,
                    name: path.to_string(),
                    s_flags: 0u16,
                    privileges: Priv::default(),
                    notify_pending: vec![],
                    async_pending: vec![],
                    minix_kerninfo_addr: None,
                };

                // allocate memory for and set the ps_strings struct in child
                let mut regs = minix_process.get_regs()?;

                // TODO: handle arguments properly: for now, we set up the process
                // with no arguments or environment strings

                // make place on the stack for the arguments,
                // and have rbx point to the ps_strings struct
                let ps_strings = PsStrings {
                    ps_argvstr: regs.rsp as u32 + 4,
                    ps_nargvstr: 1,
                    ps_envstr: 0,
                    ps_nenvstr: 0,
                };
                let ps_strings_raw: [u64; 2] = unsafe { std::mem::transmute(ps_strings) };

                regs.rsp -= std::mem::size_of::<PsStrings>() as u64;
                regs.rbx = regs.rsp;

                minix_process.write_buf(regs.rbx, &ps_strings_raw)?;

                minix_process.set_regs(regs)?;

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

    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// returns the values of registers in the traced process
    pub fn get_regs(&self) -> Result<user_regs_struct, nix::Error> {
        ptrace::getregs(self.pid)
    }

    /// sets the values of registers in the traced process
    pub fn set_regs(&self, regs: user_regs_struct) -> Result<(), nix::Error> {
        ptrace::setregs(self.pid, regs)
    }

    /// reads one word (8 bytes) from an address
    /// in the traced process's memory
    pub fn read(&self, addr: u64) -> Result<u64, nix::Error> {
        let addr = addr as *mut c_void;
        Ok(ptrace::read(self.pid, addr)? as u64)
    }

    /// reads `len` bytes from an address
    /// in the traced process's memory
    pub fn read_buf_u8(&self, addr: u64, len: usize) -> Result<Vec<u8>, nix::Error> {
        let len64 = (len + 7) / 8; // ceil(len/8)
        let mut buf: Vec<u8> = (0..len64)
            .map(|idx| {
                let addr = addr + 8 * idx as u64;
                self.read(addr).unwrap() // issue with error handling here
            })
            .flat_map(|word| {
                let bytes = unsafe { std::mem::transmute::<u64, [u8; 8]>(word) };
                std::array::IntoIter::new(bytes)
            })
            .collect();

        buf.resize(len, 0);
        Ok(buf)
    }

    /// writes one word (8 bytes) to an address
    /// in the traced process's memory
    pub fn write(&self, addr: u64, data: u64) -> Result<(), nix::Error> {
        let addr = addr as *mut c_void;
        let data = data as *mut c_void;
        unsafe {
            ptrace::write(self.pid, addr, data)?;
        }
        Ok(())
    }

    /// writes 32 bytes to an address
    /// in the traced process's memory
    pub fn write_32(&self, addr: u64, data: u32) -> Result<(), nix::Error> {
        let old_data = self.read(addr)?;

        // since we're in big endian, we want our data
        // in the more significant bits
        let data = (data as u64) << 32;
        let data = data & (old_data & 0x0000FFFF); // we want to keep the least significant half of the old value

        self.write(addr, data)
    }

    /// writes multiple words (a multiple of 8 bytes) to an address
    pub fn write_buf(&self, addr: u64, data: &[u64]) -> Result<(), nix::Error> {
        for (idx, &data) in data.iter().enumerate() {
            self.write(addr + 8 * idx as u64, data)?
        }
        Ok(())
    }

    pub fn write_buf_u8(&self, addr: u64, data: &[u8]) -> Result<(), nix::Error> {
        let len_64 = data.len() / 8;
        let data_u64 = unsafe {
            slice_from_raw_parts(data.as_ptr() as *const u64, len_64)
                .as_ref()
                .unwrap()
        };
        self.write_buf(addr, data_u64)?;
        if data.len() - 8 * len_64 > 0 {
            let mut rest = [0u8; 8];
            for (idx, &val) in data[8 * len_64..].iter().enumerate() {
                rest[idx] = val
            }
            let val_64: u64 = unsafe { std::mem::transmute(rest) };
            self.write(addr + (8 * len_64 as u64), val_64)?
        }
        Ok(())
    }

    /// reads a message the process is sending:
    /// reads 64 bytes from memory pointed to by `addr`
    pub fn read_message(&self, addr: u64) -> Result<Message, nix::Error> {
        let mut result = [0; MESSAGE_SIZE / 8];
        let result_i64: &mut [u64; MESSAGE_SIZE / 8] =
            unsafe { &mut *(result.as_mut_ptr() as *mut [u64; MESSAGE_SIZE / 8]) };
        assert_eq!(size_of_val(&result), size_of_val(result_i64));

        for (i, data) in result_i64.iter_mut().enumerate() {
            *data = self.read(addr + 8 * i as u64)?
        }

        Ok(result.into())
    }

    /// writes a message the process is waiting for:
    /// writes 64 bytes to memory pointed to by `addr`
    pub fn write_message(&self, addr: u64, message: Message) -> Result<(), nix::Error> {
        let buf: [u64; MESSAGE_SIZE / 8] = message.into();
        for (i, &data) in buf.iter().enumerate() {
            self.write(addr + 8 * i as u64, data)?;
        }
        Ok(())
    }

    /// reads one word from the address pointed to by the
    /// eip register, and returns a value corresponding
    /// to one of the relevant (to us) x86 instructions
    pub fn read_instruction(&self) -> Result<Instruction, nix::Error> {
        let regs = self.get_regs()?;
        let data: [u8; 8] = self.read(regs.rip)?.to_le_bytes();

        let result = if data[0] == 0xCD {
            Instruction::Int(data[1])
        } else {
            Instruction::Other
        };

        Ok(result)
    }

    /// cause a signal in the minix process
    /// TODO: emulate how minix handles signals, by either
    /// making pm cause a signal in a user process,
    /// or message passing in servers/drivers
    pub fn cause_signal(&self, signal: impl Into<Option<Signal>>) -> Result<(), nix::Error> {
        ptrace::cont(self.pid, signal)
    }

    /// resume stopped process
    pub fn cont(&self) -> Result<(), nix::Error> {
        ptrace::cont(self.pid, None)
    }

    /// do a Linux system call in the minix process
    pub fn _do_syscall(&self, syscall_number: u64, args: &[u64]) -> Result<u64, nix::Error> {
        // save the register values to be restored
        let old_regs = self.get_regs()?;
        let mut regs = old_regs;

        // set the eax register to the syscall number
        regs.rax = syscall_number;

        // retrieve arguments from `args`
        let mut arg_regs = [
            &mut regs.rbx,
            &mut regs.rcx,
            &mut regs.rdx,
            &mut regs.rsi,
            &mut regs.rdi,
            &mut regs.rbp,
        ];

        for (reg, &arg) in arg_regs.iter_mut().zip(args.iter()) {
            **reg = arg;
        }

        self.set_regs(regs)?;

        // write the `int 0x80` (trap to kernel) instruction
        // in place of the current instruction, saving the old
        // instruction to restore it later
        let instruction_addr = regs.rip;
        let old_instruction = self.read(instruction_addr)?;
        self.write(instruction_addr, 0x80CD)?; // 0xCD80 = int 0x80, reversed because x86 is little endian

        // execute the syscall, by using ptrace PTRACE_SYSCALL
        ptrace::syscall(self.pid, None)?; // TODO: error here is bad, because we just wrote to traced process's memory and changed register values

        // here, traced process is stopped before entering syscall,
        // so we have to wait() for it and continue it once
        // TODO: we should actually inspect the returned status (and handle possible errors) here,
        // failing to do so seems especially bad
        let _status = waitpid(self.pid, None)?;
        ptrace::syscall(self.pid, None)?;

        let _status = waitpid(self.pid, None)?;

        let after_regs = self.get_regs()?;

        // we are now after the syscall: restore old instructions and registers
        self.write(instruction_addr, old_instruction)?;
        self.set_regs(old_regs)?;

        // the result of syscall is stored in eax
        Ok(after_regs.rax)
    }

    /// attaches memory represented by the SharedMemory struct
    /// in the process at the given address.
    /// It's important that the SharedMemory struct is created
    /// before the process we're attaching to (since we rely on the
    /// file descriptor of the shared memory)
    pub fn attach_shared(&self, shared: &SharedMemory, addr: u64) -> Result<(), nix::Error> {
        // not going to work, mmap uses too many arguments
        let syscall_number = 90; // mmap syscall number
        let mmap_args: [u32; 6] = [
            addr as u32,       // address to map to
            shared.len as u32, // length of mapped memory
            0x1,               // file access, set to PROT_READ
            0x1 | 0x10,        // mmap flags, set to MAP_SHARED | MAP_FIXED
            shared.fd as u32,  // file descriptor to be mapped
            0,                 // offset
        ];

        // get the mmap args as a byte slice
        let len = size_of_val(&mmap_args);
        let raw_args = unsafe { std::slice::from_raw_parts(mmap_args.as_ptr() as *const u8, len) };

        // get the stack address
        let regs = self.get_regs()?;
        let args_addr = regs.rsp - len as u64;

        // write the mmap_args to the process's stack
        // the can process use the values stored _below_ rsp
        // we should save them and restore after the syscall
        let old_data = self.read_buf_u8(args_addr, len)?;
        self.write_buf_u8(args_addr, raw_args)?;

        // do mmap system call, with the arguments stored in a struct
        // on the top of the stack
        // TODO: alignment of the args may be wrong, depending on rsp
        let result = self._do_syscall(syscall_number, &[args_addr])?;

        // mmap failed or mapped to wrong address (for some reason?)
        if result as u32 != addr as u32 {
            // TODO: just return an error
            panic!("Error in mmap in child process: {}", result as i32);
        }

        // restore the old values on the stack
        self.write_buf_u8(args_addr, &old_data)?;

        Ok(())
    }
}

// since a MinixProcess value corresponds to a running process,
// we should probably terminate the process when the value is dropped.
impl Drop for MinixProcess {
    fn drop(&mut self) {
        // some thought should probably be given here,
        // let's just kill the process for now.
        // errors here can probably be safely ignored?
        let _ = kill(self.pid, Some(Signal::SIGKILL));
    }
}

/// A Minix structure containing information about arguments
/// and environment variables a process was started with.
/// A Minix process needs the ebx register to point to this
/// structure at startup
#[repr(C)]
struct PsStrings {
    ps_argvstr: u32,
    ps_nargvstr: u32,
    ps_envstr: u32,
    ps_nenvstr: u32,
}

/// This module contains bits for the s_flags field
#[allow(dead_code)]
pub mod priv_flags {
    pub const PREEMPTIBLE: u16 = 0x002;
    pub const BILLABLE: u16 = 0x004;
    pub const DYN_PRIV_ID: u16 = 0x008;

    pub const SYS_PROC: u16 = 0x010;
    pub const CHECK_IO_PORT: u16 = 0x020;
    pub const CHECK_IRQ: u16 = 0x040;
    pub const CHECK_MEM: u16 = 0x080;
    pub const ROOT_SYS_PROC: u16 = 0x100;
    pub const VM_SYS_PROC: u16 = 0x200;
    pub const LU_SYS_PROC: u16 = 0x400;
    pub const RST_SYS_PROC: u16 = 0x800;
}

#[cfg(test)]
mod tests {
    use nix::sys::wait::WaitStatus;

    use super::MinixProcess;

    #[test]
    fn do_syscall_test() {
        let path = format!("{}/syscall", env!("CARGO_MANIFEST_DIR"));
        let process = MinixProcess::spawn(&path).unwrap();

        match nix::sys::wait::wait().unwrap() {
            WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP) => {
                let written = process._do_syscall(4, &[]).unwrap(); // sys_write syscall number
                assert_eq!(5, written);
            }
            _ => panic!("process wasn't stopped by SIGTRAP"),
        };

        process.cont().unwrap();
        let status = nix::sys::wait::wait().unwrap();
        if let WaitStatus::Exited(_, 0) = status {
            return;
        }
        panic!("wrong exit");
    }

    #[test]
    fn attach_shared_test() {
        use crate::utils::SharedMemory;

        let shared_mem = SharedMemory::new("test", std::mem::size_of::<i32>()).unwrap();
        shared_mem.write(0, &42i32).unwrap();

        let path = format!("{}/attach", env!("CARGO_MANIFEST_DIR"));
        let process = MinixProcess::spawn(&path).unwrap();

        let addr = 0xf1002000u32;

        match nix::sys::wait::wait().unwrap() {
            WaitStatus::Stopped(_, nix::sys::signal::Signal::SIGTRAP) => {
                process.attach_shared(&shared_mem, addr as u64).unwrap();
            }
            _ => panic!("process wasn't stopped by SIGTRAP"),
        };

        process.cont().unwrap();
        let status = nix::sys::wait::wait().unwrap();
        if let WaitStatus::Exited(_, 42) = status {
            return;
        }
        panic!("wrong exit");
    }
}
