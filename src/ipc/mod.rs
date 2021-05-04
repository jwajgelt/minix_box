#[allow(dead_code)]
mod asyn;

use crate::utils::{endpoint, Endpoint, Message, NOTIFY_MESSAGE};
use crate::utils::{
    minix_errno::{self, EDEADSRCDST, EINVAL, ENOTREADY, OK},
    MinixProcess,
};
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

    let call_nr = regs.rcx;

    // TODO: if call is SEND, SENDNB, SENDREC or NOTIFY, verify
    // that the caller is allowed to send to the given destination
    // else return ECALLDENIED in process.

    // TODO: check if the process has privileges for the requested call.
    // Calls to the kernel may only be SENDREC because tasks always reply.
    // If illegal, return ETRAPDENIED in process.

    // the ecx register contains the type of ipc call
    let result = match call_nr {
        ipcconst::SEND..=ipcconst::SENDNB => do_sync_ipc(
            caller_endpoint,
            call_nr,
            regs.rax as Endpoint,
            process_table,
        )?,
        ipcconst::SENDA => {
            asyn::do_senda(caller_endpoint, regs.rbx, regs.rax, process_table)?
        }
        ipcconst::MINIX_KERNINFO => {
            // check if process has the `minix_kerninfo` struct already mapped
            // and if not, map it to the process's memory
            if process_table[caller_endpoint].minix_kerninfo_addr.is_none() {
                let usermapped_mem = &process_table.usermapped_mem;
                process_table[caller_endpoint]
                    .attach_shared(usermapped_mem, crate::utils::SHARED_BASE_ADDR as u64)
                    .unwrap();
                process_table[caller_endpoint].minix_kerninfo_addr =
                    Some(crate::utils::SHARED_BASE_ADDR);
            }

            // ebx is the return struct ptr
            regs.rbx = process_table[caller_endpoint].minix_kerninfo_addr.unwrap() as u64;
            regs.rax = if regs.rbx == 0 {
                minix_errno::EDEADSRCDST
            } else {
                minix_errno::OK
            } as u64;
            process_table[caller_endpoint].set_regs(regs)?;

            // run the process
            process_table[caller_endpoint].cont()?;
            return Ok(());
        }
        _ => {
            // invalid call number - return EBADCALL in process
            todo!("invalid ipc call number")
        }
    };

    // if the caller doesn't have any ipc state set,
    // set the ipc call return value and resume it
    if let ProcessState::Running = process_table[caller_endpoint].state {
        set_return_value(&process_table[caller_endpoint], result)?;
        process_table[caller_endpoint].cont()?;
    };

    Ok(())
}

fn do_sync_ipc(
    caller: Endpoint,
    call_nr: u64,
    dest_src: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // ANY endpoint is allowed only for the RECEIVE call
    if dest_src == endpoint::ANY && call_nr != ipcconst::RECEIVE {
        // return EINVAL in process
        return Ok(EINVAL);
    }

    // check the source / destination is valid
    if dest_src != endpoint::ANY && process_table.get(dest_src).is_none() {
        // return EDEADSRCDST in process
        todo!(
            "Handle invalid endpoints. Endpoint: {}, call: {}",
            dest_src,
            call_nr
        );
    }

    let result = match call_nr {
        ipcconst::SEND => do_send(caller, dest_src, process_table, false)?,
        ipcconst::RECEIVE => {
            process_table[caller].reply_pending = false;
            do_receive(caller, dest_src, process_table)?
        }
        ipcconst::SENDREC => do_sendrec(caller, dest_src, process_table)?,
        ipcconst::NOTIFY => do_notify(caller, dest_src, process_table)?,
        ipcconst::SENDNB => do_send(caller, dest_src, process_table, true)?,
        _ => unreachable!(),
    };

    Ok(result)
}

fn do_send(
    caller: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
    non_blocking: bool,
) -> Result<i32, nix::Error> {
    let addr = process_table[caller].get_regs()?.rbx;
    let mut message = process_table[caller].read_message(addr)?; // TODO: return EFAULT in child if read is not successful

    // check if `dst` is blocked waiting for this message
    if will_receive(caller, dst, process_table) {
        // TODO: here, Minix checks if message comes from the kernel
        // and does some magic if so - might want to think this through

        // set the source of the message
        message.source = caller;

        // TODO: minix sets call status in receiver here

        let receiver = &mut process_table[dst];
        // write the message to the receiver's memory
        let addr = receiver.get_regs()?.rbx;

        receiver.write_message(addr, message)?;

        println!("{:016x?}", receiver.read_message(addr).unwrap());

        // TODO: set the IPC status in receiver

        // unset the `RECEIVING` status in `dst`
        receiver.state = match receiver.state {
            ProcessState::SendReceiving(dst) => ProcessState::Sending(dst),
            ProcessState::Receiving(_) => {
                receiver.cont()?;
                ProcessState::Running
            }
            _ => unreachable!("Receiver has to be in either RECEIVE or SENDRECEIVE"),
        }
    } else {
        // return ENOTREADY
        if non_blocking {
            return Ok(ENOTREADY);
        }

        // TODO: Minix checks for a possible deadlock,
        // and returns ELOCKED in sender if detected

        // set the sender's state as SENDING to the receiver
        // this blocks the sender, waiting for the receiver
        // to call `receive`
        let sender = &mut process_table[caller];
        sender.state = ProcessState::Sending(dst);

        // add `caller` to `dst` queue
        process_table[dst].queue.insert(caller, message);
    }

    Ok(OK)
}

// TODO: this is, so far, very naive - we don't handle NOTIFYs, async sends,
// we don't check flags for non-blocking, we don't set the return value
fn do_receive(
    caller: Endpoint,
    src: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // caller is in SENDREC, and destination wasn't ready:
    // set the state as `SENDRECEIVING` from `src` and return
    if let ProcessState::Sending(dst) = process_table[caller].state {
        assert!(src == dst);
        process_table[caller].state = ProcessState::SendReceiving(src);
        return Ok(OK);
    };

    // check if there are pending notifications, except for SENDREC
    if !process_table[caller].reply_pending {
        for (index, &src) in process_table[caller].notify_pending.iter().enumerate() {
            if can_receive(caller, src) {
                let receiver = &mut process_table[caller];
                receiver.notify_pending.remove(index);
                let msg = build_notify_message(src);
                let addr = receiver.get_regs()?.rbx;
                receiver.write_message(addr, msg)?;
                return Ok(OK);
            }
        }
    }

    // TODO: check for pending asynchronous messages

    // look on the queue for an appropriate message
    if let Some((sender, mut message)) = process_table[caller]
        .queue
        .get(|sender| can_receive(src, sender))
    {
        // unset the `SENDING` state in sender
        let sender = &mut process_table[sender];
        sender.state = match sender.state {
            ProcessState::SendReceiving(dst) => ProcessState::Receiving(dst),
            ProcessState::Sending(_) => {
                // since we're setting the state as 'running',
                // we should resume the process
                sender.cont()?;
                ProcessState::Running
            }
            _ => unreachable!("Sender has to be in either SEND or SENDRECEIVE"),
        };

        // set the source of the message
        message.source = caller;

        // write the message to receiver
        let receiver = &process_table[caller];
        let addr = receiver.get_regs()?.rbx;
        receiver.write_message(addr, message)?;

        println!("{:016x?}", receiver.read_message(addr).unwrap());

        // TODO: set the IPC status here
        // status is stored in the ebx register
        return Ok(OK);
    }

    // Minix checks if `receive` is non-blocking
    // and if so, returns ENOTREADY instead of blocking.
    // However, `mini_receive` is only called with empty
    // flags, so this never actually happens.

    // TODO: before block, check for deadlocks

    // set the caller as `RECEIVING` from `src`
    process_table[caller].state = ProcessState::Receiving(src);
    Ok(OK)
}

fn do_sendrec(
    caller: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    process_table[caller].reply_pending = true;
    let send_result = do_send(caller, dst, process_table, false)?;
    if send_result != OK {
        return Ok(send_result);
    }
    do_receive(caller, dst, process_table)
}

fn do_notify(
    caller: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    if process_table.get(dst).is_none() {
        // TODO: return EDEADSRCDST from ipc call
        return Ok(EDEADSRCDST);
    }

    if !will_receive(caller, dst, process_table) {
        process_table[dst].notify_pending.push(caller);
        return Ok(OK);
    }

    // TODO: check the 'sendrecing' flag (`MF_REPLY_PEND`)
    // once that's implemented

    let msg = build_notify_message(caller);
    let receiver = &mut process_table[dst];
    let addr = receiver.get_regs()?.rbx;
    receiver.write_message(addr, msg)?;

    // set the receiver status to running and run it

    // TODO: set the IPC status in receiver

    // unset the `RECEIVING` status in `dst`
    receiver.state = match receiver.state {
        ProcessState::Receiving(_) => {
            receiver.cont()?;
            ProcessState::Running
        }
        _ => unreachable!("Notify receiver has to be in either RECEIVE"),
    };

    Ok(OK)
}

// sets the rax register to be the return value
// of the ipc call
fn set_return_value(process: &MinixProcess, value: i32) -> Result<(), nix::Error> {
    let mut regs = process.get_regs()?;
    regs.rax = value as u64;
    process.set_regs(regs)
}

// assumes `src` and `dst` are valid endpoints
// and will panic otherwise
fn will_receive(src: Endpoint, dst: Endpoint, process_table: &MinixProcessTable) -> bool {
    if let ProcessState::Receiving(receive_e) = process_table[dst].state {
        can_receive(receive_e, src)
    } else {
        false
    }
}

// TODO: add all the things the CANRECEIVE macro actually does
fn can_receive(receive_e: Endpoint, sender: Endpoint) -> bool {
    assert!(sender != endpoint::ANY);
    // Minix checks allow_ipc_filtered_msg() here
    receive_e == endpoint::ANY || receive_e == sender
}

// TODO: fill the notify with payload when appropriate
fn build_notify_message(src: Endpoint) -> Message {
    Message {
        m_type: NOTIFY_MESSAGE,
        source: src,
        payload: [0; 14],
    }
}
