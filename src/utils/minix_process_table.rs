use array_init::array_init;
use nix::unistd::Pid;
use std::collections::BTreeMap;

use super::MinixProcess;

type Endpoint = i32;

const MAX_PROCESSES: usize = 256;

/// a struct containing all running Minix processes, indexed by their endpoints
pub struct MinixProcessTable {
    /// the array of all running Minix processes
    table: [Option<MinixProcess>; MAX_PROCESSES],
    /// a map mapping (Linux) pids to indices in the `table` array
    pid_map: BTreeMap<Pid, usize>,
}

impl MinixProcessTable {
    pub fn new() -> Self {
        Self {
            table: array_init(|_| None),
            pid_map: BTreeMap::new(),
        }
    }

    pub fn get(&self, endpoint: Endpoint) -> Option<&MinixProcess> {
        let idx = endpoint as usize;
        self.table[idx].as_ref()
    }

    pub fn get_mut(&mut self, endpoint: Endpoint) -> Option<&mut MinixProcess> {
        let idx = endpoint as usize;
        self.table[idx].as_mut()
    }

    /// returns a reference to the MinixProcess struct
    /// with the given (Linux) pid
    pub fn get_by_pid(&self, pid: Pid) -> Option<&MinixProcess> {
        let idx = *self.pid_map.get(&pid)?;
        self.table[idx].as_ref()
    }

    /// returns a mutable reference to the MinixProcess struct
    /// with the given (Linux) pid
    pub fn get_mut_by_pid(&mut self, pid: Pid) -> Option<&mut MinixProcess> {
        let idx = *self.pid_map.get(&pid)?;
        self.table[idx].as_mut()
    }

    pub fn insert(&mut self, proc: MinixProcess, endpoint: Endpoint) -> Result<(), ()> {
        let idx = endpoint as usize;
        if self.table[idx].is_some() {
            return Err(());
        }
        let pid = proc.pid();
        self.table[idx] = Some(proc);
        self.pid_map.insert(pid, idx);
        Ok(())
    }

    pub fn remove(&mut self, endpoint: Endpoint) -> Option<MinixProcess> {
        let idx = endpoint as usize;
        let process = self.table[idx].take();
        if let Some(process) = process.as_ref() {
            self.pid_map.remove(&process.pid());
        };
        process
    }
}

// don't think this is necessary, but may be in the future,
// if MinixProcessTable implements some more complex logic
impl Drop for MinixProcessTable {
    fn drop(&mut self) {
        for i in 0..MAX_PROCESSES {
            self.remove(i as Endpoint);
        }
    }
}
