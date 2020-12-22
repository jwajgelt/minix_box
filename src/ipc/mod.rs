use nix::unistd::Pid;

use crate::utils::MinixProcessTable;

mod ipcconst {
    pub const SEND: u64 = 1;
    pub const RECEIVE: u64 = 2;
    pub const SENDREC: u64 = 3;
}

pub fn handle_ipc(pid: Pid, process_table: &mut MinixProcessTable) -> Result<(), nix::Error> {
    let process = process_table.get_by_pid(pid).unwrap();
    let regs = process.get_regs()?;

    // the ecx register contains the type of ipc call
    match regs.rcx {
        ipcconst::SEND => {
            let message = process.retrieve_message().unwrap(); // TODO: if error here, cause error in process
            println!("{:#010x?}", &message);
            todo!()
        }
        ipcconst::RECEIVE => {
            let message = process.retrieve_message().unwrap(); // TODO: if error here, cause error in process
            println!("{:#010x?}", &message);
            todo!()
        }
        ipcconst::SENDREC => {
            todo!()
        }
        _ => {
            // TODO: handle everything
        }
    }

    Ok(())
}
