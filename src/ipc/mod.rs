use endpoint::ANY;

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
    if dest_src != ANY && process_table.get(dest_src).is_none() {
        // return EDEADSRCDST in process
        todo!("Handle invalid endpoints.");
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
        ipcconst::SEND => do_send(caller_endpoint, dest_src, process_table)?,
        ipcconst::RECEIVE => do_receive(caller_endpoint, dest_src, process_table)?,
        ipcconst::SENDREC => do_sendrec(caller_endpoint, dest_src, process_table)?,
        _ => {
            // invalid call number - return EBADCALL in process
            todo!()
        }
    };

    // if the caller doesn't have any ipc state set,
    // resume it
    if let ProcessState::Running = process_table[caller_endpoint].state {
        process_table[caller_endpoint].cont()?;
    };

    Ok(())
}

fn do_send(
    caller: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    // check if `dst` is blocked waiting for this message
    if will_receive(caller, dst, process_table) {
        // TODO: here, Minix checks if message comes from the kernel
        // and does so magic if so - might want to think this through
        let mut message = process_table[caller].read_message()?; // TODO: return EFAULT in child if read is not successful

        // set the source of the message
        // TODO: do the messages correctly
        message[0] = caller as u64;

        // TODO: minix sets call status in receiver here

        let receiver = &mut process_table[dst];
        // write the message to the receiver's memory
        receiver.write_message(message).unwrap();

        // TODO: set the IPC status in receiver

        // unset the `RECEIVING` status in `dst`
        receiver.state = match receiver.state {
            ProcessState::SendReceiving(dst) => ProcessState::Sending(dst),
            ProcessState::Receiving(_) => {
                receiver.cont().unwrap();
                ProcessState::Running
            }
            _ => unreachable!("Receiver has to be in either RECEIVE or SENDRECEIVE"),
        }
    } else {
        // TODO: Minix checks for a possible deadlock,
        // and returns ELOCKED in sender if detected

        // set the sender's state as SENDING to the receiver
        // this blocks the sender, waiting for the receiver
        // to call `receive`
        let sender = &mut process_table[caller];
        sender.state = ProcessState::Sending(dst);

        // TODO: Minix uses a queue to manage incoming messages
        // here, we should add `caller` to `dst` queue
    }

    Ok(())
}

// assumes `src` and `dst` are valid endpoints
// and will panic otherwise
fn will_receive(src: Endpoint, dst: Endpoint, process_table: &MinixProcessTable) -> bool {
    let receiver_state = process_table[dst].state;

    match receiver_state {
        ProcessState::Receiving(endpoint::ANY) => true,
        ProcessState::Receiving(sender) => sender == src,
        ProcessState::SendReceiving(sender) => sender == src,
        _ => false,
    }
}

// TODO: this is, so far, very naive - we don't expect ANY as an endpoint, we don't handle
// NOTIFYs, async sends, we don't check flags for non-blocking, we don't do message queues,
// we don't set the return value
fn do_receive(
    caller: Endpoint,
    src: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    // caller is in SENDREC, and destination wasn't ready:
    // set the state as `SENDRECEIVING` from `src` and return
    if let ProcessState::Sending(dst) = process_table[caller].state {
        assert!(src == dst);
        process_table[caller].state = ProcessState::SendReceiving(src);
        return Ok(());
    };

    // TODO: check if there are pending notifications, except for SENDREC

    // TODO: check for pending asynchronous messages

    // TODO: Minix uses a caller queue for pending messages,
    // look on the queue for an appropriate message
    if can_receive(src, src, caller, process_table) {
        let sender = &mut process_table[src];
        let mut message = sender.read_message().unwrap();

        // set the source of the message
        // TODO: do the messages correctly
        message[0] = caller as u64;
        // unset the `SENDING` state in sender
        sender.state = match sender.state {
            ProcessState::SendReceiving(dst) => ProcessState::Receiving(dst),
            ProcessState::Sending(_) => {
                // since we're setting the state as 'running',
                // we should resume the process
                sender.cont().unwrap();
                ProcessState::Running
            }
            _ => unreachable!("Sender has to be in either SEND or SENDRECEIVE"),
        };

        // write the message to receiver
        let receiver = &process_table[caller];
        receiver.write_message(message).unwrap();
        // TODO: set the IPC status here
        return Ok(());
    }

    // TODO: check if `receive` is non-blocking
    // if so, return ENOTREADY instead of blocking

    // TODO: before block, check for deadlocks

    // set the caller as `RECEIVING` from `src`
    process_table[caller].state = ProcessState::Receiving(src);
    Ok(())
}

// TODO: read what the CANRECEIVE macro actually does
fn can_receive(
    src: Endpoint,
    sender: Endpoint,
    _caller: Endpoint,
    _process_table: &MinixProcessTable,
) -> bool {
    assert!(sender != ANY);
    src == ANY || src == sender
}

fn do_sendrec(
    caller: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    do_send(caller, dst, process_table)?;
    // TODO: set a flag stopping notifies in receive
    do_receive(caller, dst, process_table)
}
