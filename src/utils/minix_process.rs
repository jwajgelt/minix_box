use super::{message_queue::MessageQueue, Endpoint, MESSAGE_SIZE};
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

                let mut minix_process = Self {
                    pid: child,
                    state: ProcessState::Running,
                    queue: MessageQueue::new(),
                };

                // allocate memory for and set the ps_strings struct in child
                let mut regs = minix_process.get_regs()?;

                // TODO: handle arguments properly: for now, we set up the process
                // with no environment strings

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
    pub fn set_regs(&mut self, regs: user_regs_struct) -> Result<(), nix::Error> {
        ptrace::setregs(self.pid, regs)
    }

    /// reads one word (8 bytes) from an address
    /// in the traced process's memory
    pub fn read(&self, addr: u64) -> Result<i64, nix::Error> {
        let addr = addr as *mut c_void;
        ptrace::read(self.pid, addr)
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

    /// writes multiple words (a multiple of 8 bytes) to an address
    pub fn write_buf(&self, addr: u64, data: &[u64]) -> Result<(), nix::Error> {
        for (idx, &data) in data.iter().enumerate() {
            self.write(addr + 8 * idx as u64, data)?
        }
        Ok(())
    }

    /// reads a message the process is sending:
    /// reads 64 bytes from memory pointed to
    /// by the ebx register
    pub fn read_message(&self) -> Result<[u64; MESSAGE_SIZE / 8], nix::Error> {
        let mut result = [0; MESSAGE_SIZE / 8];
        let result_i64: &mut [i64; MESSAGE_SIZE / 8] =
            unsafe { &mut *(result.as_mut_ptr() as *mut [i64; MESSAGE_SIZE / 8]) };
        assert_eq!(size_of_val(&result), size_of_val(result_i64));

        let regs = self.get_regs()?;

        let addr = regs.rbx;
        for (i, data) in result_i64.iter_mut().enumerate() {
            *data = self.read(addr + 8 * i as u64)?
        }

        Ok(result)
    }

    /// writes a message the process is waiting for:
    /// writes 64 bytes to memory pointed to
    /// by the ebx register
    pub fn write_message(&self, message: [u64; MESSAGE_SIZE / 8]) -> Result<(), nix::Error> {
        let regs = self.get_regs()?;

        let addr = regs.rbx;
        for (i, &data) in message.iter().enumerate() {
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
    pub fn _do_syscall(&mut self, syscall_number: u64, args: &[u64]) -> Result<u64, nix::Error> {
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

        // write the `int 0x80` (trap to kernel) instruction
        // in place of the current instruction, saving the old
        // instruction to restore it later
        let instruction_addr = regs.rip;
        let old_instruction = unsafe { std::mem::transmute(self.read(instruction_addr)?) };
        self.write(instruction_addr, 0xCD80)?; // 0xCD80 = int 0x80

        // execute the syscall, by using ptrace PTRACE_SYSCALL
        ptrace::syscall(self.pid, None)?; // TODO: error here is bad, because we just wrote to traced process's memory

        // here, traced process is stopped before entering syscall,
        // so we have to wait() for it and continue it once
        let _status = waitpid(self.pid, None)?;
        ptrace::syscall(self.pid, None)?;

        let _status = waitpid(self.pid, None)?;

        // we are now after the syscall: restore old instructions and registers
        self.write(instruction_addr, old_instruction)?;
        self.set_regs(old_regs)?;

        // the result of syscall is stored in eax
        Ok(regs.rax)
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
