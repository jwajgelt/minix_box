use crate::utils::Endpoint;
use crate::utils::{MinixProcessTable, ProcessState};

#[allow(dead_code)]
mod ipcconst {
    // ipc call numbers, defined in minix/ipcconst.h
    pub const SEND: u64 = 1; // blocking send
    pub const RECEIVE: u64 = 2; // blocking receive
    pub const SENDREC: u64 = 3; // SEND + RECEIVE
    pub const NOTIFY: u64 = 4; // asynchronous notify
    pub const SENDNB: u64 = 5; // nonblocking send
    pub const MINIX_KERNINFO: u64 = 6; // request kernel info structure
    pub const SENDA: u64 = 16; // asynchronous send
}

#[allow(dead_code)]
mod endpoint {
    // special endpoints defined in minix/endpoint.h
    use crate::utils::Endpoint;
    const ENDPOINT_GENERATION_SHIFT: Endpoint = 15;
    const ENDPOINT_GENERATION_SIZE: Endpoint = 1 << ENDPOINT_GENERATION_SHIFT;
    const ENDPOINT_SLOT_TOP: Endpoint = ENDPOINT_GENERATION_SIZE - 1023; // ENDPOINT_GENERATION_SIZE - MAX_NR_TASKS

    pub const ANY: Endpoint = ENDPOINT_SLOT_TOP - 1;
    pub const NONE: Endpoint = ENDPOINT_SLOT_TOP - 2;
    pub const SELF: Endpoint = ENDPOINT_SLOT_TOP - 3;
}

/// handles the ipc calls from processes
pub fn do_ipc(
    caller_endpoint: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    // in Minix, this is implemented in kernel/proc.c

    let process = process_table.get_mut(caller_endpoint).unwrap();
    let mut regs = process.get_regs()?;
    // advance the instruction pointer to the next instruction
    regs.rip += 2;
    process.set_regs(regs).unwrap();

    let dest_src = regs.rax as Endpoint;
    let call_nr = regs.rcx;

    // ANY endpoint is allowed only for the RECEIVE call
    if dest_src == endpoint::ANY && call_nr != ipcconst::RECEIVE {
        // return EINVAL in process
        todo!();
    }

    // check the source / destination is valid
    if process_table.get(dest_src).is_none() {
        // return EDEADSRCDST in process
        todo!();
    }

    // TODO: if call is SEND, SENDNB, SENDREC or NOTIFY, verify
    // that the caller is allowed to send to the given destination
    // else return ECALLDENIED in process.

    // TODO: check if the process has privileges for the requested call.
    // Calls to the kernel may only be SENDREC because tasks always reply.
    // If illegal, return ETRAPDENIED in process.

    // the ecx register contains the type of ipc call
    match call_nr {
        // TODO: handle everything
        ipcconst::SEND => do_send(caller_endpoint, dest_src, process_table),
        ipcconst::RECEIVE => do_receive(dest_src, caller_endpoint, process_table),
        ipcconst::SENDREC => do_sendrec(caller_endpoint, dest_src, process_table),
        _ => {
            // invalid call number - return EBADCALL in process
            todo!()
        }
    }
}

fn do_send(
    from: Endpoint,
    to: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    let message = process_table[from].read_message().unwrap(); // if error here, cause error in process?

    let receiver_state = process_table[to].state;

    match receiver_state {
        // TODO: or ANY
        ProcessState::Receiving(sender) if sender == from => {
            // write the message to the receiver's memory
            // and resume it
            let receiver = &mut process_table[to];
            receiver.write_message(message).unwrap(); // TODO: error handling everywhere
            receiver.state = ProcessState::Running;
            receiver.cont().unwrap();

            // resume the sender
            let sender = &process_table[from];
            sender.cont().unwrap();
            return Ok(());
        }
        _ => {
            // set the sender's state as SENDING to the receiver
            let sender = &mut process_table[from];
            sender.state = ProcessState::Sending(to);
            // we don't resume the sender, since it's supposed to be blocked on send
            return Ok(());
        }
    }
}

// TODO: this is, so far, very naive - we don't expect ANY as an endpoint, we don't handle
// NOTIFYs, async sends, we don't check flags for non-blocking, we don't do message queues,
// we don't set the return value
fn do_receive(
    from: Endpoint,
    to: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    // TODO: handle the case of send_endpoint being ANY
    let sender_state = process_table[from].state;
    match sender_state {
        ProcessState::Sending(receiver) if receiver == to => {
            // read the message from the sender's memory,
            // write to to receiver's memory and resume both processes
            let sender = &mut process_table[from];
            let message = sender.read_message().unwrap(); // TODO: error handling
            sender.state = ProcessState::Running;
            sender.cont().unwrap();

            let receiver = &process_table[to];
            receiver.write_message(message).unwrap();
            receiver.cont().unwrap();

            // TODO: before returning, we should set the return value in
            // receiver as well - it should be in the ebx register
            return Ok(());
        }
        _ => {
            // set the receiver's state as RECEIVING from the sender
            let receiver = &mut process_table[to];
            receiver.state = ProcessState::Receiving(from);
            // don't resume the receiver here, since it's supposed to be blocked on receive
            return Ok(());
        }
    }
}

fn do_sendrec(
    _from: Endpoint,
    _to: Endpoint,
    _process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    todo!()
}
// #[repr(C)]
// struct Message {
//     pub m_source: Endpoint,
//     pub m_type: i32,
//     pub payload: [u8; 56]
// }
