use std::{ffi::CString, os::raw::c_int};

use nix::{
    fcntl::{self, fcntl, FdFlag, OFlag},
    sys::mman::shm_open,
    sys::mman::shm_unlink,
    sys::stat::Mode,
    unistd::ftruncate,
};

/// an object representing a POSIX shared memory,
/// inherited (as an open file descriptor `fd`)
/// by all child processes
pub struct SharedMemory {
    pub len: usize,
    pub fd: c_int,
    name: CString,
}

impl SharedMemory {
    pub fn new(name: &str, len: usize) -> Result<Self, nix::Error> {
        let name = CString::new(name).unwrap();

        let fd = shm_open(
            name.as_c_str(),
            OFlag::O_RDWR | OFlag::O_CREAT,
            Mode::S_IRWXU,
        )?;

        let result = Self { len, fd, name };

        // unset the FD_CLOEXEC file descriptor flag
        let mut flags = FdFlag::from_bits(fcntl::fcntl(fd, fcntl::F_GETFD)?).unwrap();
        flags.set(FdFlag::FD_CLOEXEC, false);
        fcntl(fd, fcntl::F_SETFD(flags))?;

        ftruncate(fd, len as i64)?;

        Ok(result)
    }

    pub fn write<T: Sized>(&self, offset: usize, val: &T) -> Result<(), nix::Error> {
        let mut written = 0;
        let len = std::mem::size_of::<T>();
        let buf: &[u8] = unsafe { std::slice::from_raw_parts(val as *const T as *const u8, len) };
        nix::unistd::lseek(self.fd, offset as i64, nix::unistd::Whence::SeekSet)?;
        while written < len {
            written += nix::unistd::write(self.fd, &buf[written..])?
        }
        Ok(())
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        let _ = shm_unlink(self.name.as_c_str());
    }
}
