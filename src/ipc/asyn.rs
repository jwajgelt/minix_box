use std::mem::size_of;

use crate::{
    ipc::can_receive, 
    utils::{
        Message, 
        MinixProcess, 
        MinixProcessTable, 
        endpoint::{ASYNCM, Endpoint}, 
        minix_errno::{EAGAIN, ECALLDENIED, EINVAL, ESRCH, OK}
    }
};

/// bits for the `flags` field
#[allow(dead_code)]
mod asyn_flags {
    pub const AMF_EMPTY: u32 = 0; // slot is not in use
    pub const AMF_VALID: u32 = 1; // slot contains a message
    pub const AMF_DONE: u32 = 2; // kernel has processed the message
                                 // the result is stored in `result`
    pub const AMF_NOTIFY: u32 = 4; // send a notification when AMF_DONE is set
    pub const AMF_NOREPLY: u32 = 8; // not a reply message for a SENDREC
    pub const AMF_NOTIFY_ERR: u32 = 16; // send a notification on error
}

use asyn_flags::*;

#[repr(C)]
struct AsynMsg {
    flags: u32,
    dst: Endpoint,
    result: i32,
    msg: Message,
}

pub fn has_pending_asend(
    _dst: Endpoint,
    _src: Endpoint,
    _process_table: &mut MinixProcessTable,
) -> Result<bool, nix::Error> {
    unimplemented!()
}

pub fn try_one(
    src: Endpoint,
    dst: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // TODO: if `src` is not a system process, 
    // return EPERM
    let size = process_table[src].privileges.s_asynsize;
    let table_addr = process_table[src].privileges.s_asyntab;

    // clear the pending message
    for (idx, &endpoint) in process_table[dst].async_pending.iter().enumerate() {
        if endpoint == src {
            process_table[dst].async_pending.remove(idx);
            break;
        }
    }

    if size == 0 { return Ok(EAGAIN); }
    // here, Minix checks if the endpoint in `priv`
    // is the same as the `src`. We don't set the 
    // endpoint in `priv` (yet), so we skip this check
    if !may_asynsend_to(src, dst, process_table)? { return Ok(ECALLDENIED); }

    let mut do_notify = false;
    let mut done = true;
    let mut r = EAGAIN;

    for i in 0..size {
        let addr = table_addr + i*size_of::<AsynMsg>() as u32;
        let mut tabent = read_asynmsg(addr as u64, &process_table[src])?;

        // skip empty entries
        if tabent.flags == AMF_EMPTY { continue; }

        if tabent.flags & !(AMF_VALID|AMF_DONE|AMF_NOTIFY|AMF_NOREPLY|AMF_NOTIFY_ERR) != 0 {
            // `flags` field must contain only valid bits
            r = EINVAL;
        } else if tabent.flags & AMF_VALID == 0 {
            // must contain a message
            r = EINVAL;
        } else if tabent.flags & AMF_DONE != 0 { 
            // already done processing
            continue; 
        }

        done = false;

        if r != EINVAL {
            // we're only interested in messages 
            // directed to `dst`
            if tabent.dst != dst { continue; }

            if !can_receive(dst, src) { continue; }

            // if this is not a reply to SENDREC, and receiver is waiting
            // for a reply, this is not the message it's waiting for and 
            // should be delivered later
            if (tabent.flags & AMF_NOREPLY != 0) && (process_table[dst].reply_pending) {
                continue;
            }

            // destination is ready to receive the message; deliver it
            r = OK;
            let dst = &process_table[dst];
            let mut message = tabent.msg;
            message.source = src;
            let addr = dst.get_regs()?.rbx;
            dst.write_message(addr, message)?;
        }

        tabent.result = r;
        tabent.flags = tabent.flags | AMF_DONE;
        if tabent.flags & AMF_NOTIFY != 0 { do_notify = true; }
        else if r != OK && tabent.flags & AMF_NOTIFY_ERR != 0 { do_notify = true; }

        write_asynmsg(tabent, addr as u64, &process_table[src])?;

        break;
    }

    if do_notify {
        super::do_notify(ASYNCM, src, process_table)?;
    }

    if done {
        let privileges = &mut process_table[src].privileges;
        privileges.s_asyntab = (-1i32) as u32;
        privileges.s_asynsize = 0;
    } else {
        process_table[dst].notify_pending.push(src);
    }

    Ok(r)
}

pub fn try_async(
    caller: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    let async_pending = process_table[caller].async_pending.clone();

    for src in async_pending {
        let result = try_one(src, caller, process_table)?;
        if result == OK {
            return Ok(result);
        }
    }

    Ok(ESRCH)
}

pub fn do_senda(
    caller: Endpoint,
    _asynmsg_addr: u64,
    size: u64,
    process_table: &mut MinixProcessTable,
) -> Result<i32, nix::Error> {
    // TODO: check if caller is a system process

    let privileges = &mut process_table[caller].privileges;
    privileges.s_asyntab = (-1i32) as u32;
    privileges.s_asynsize = 0;
    privileges.s_asynendpoint = caller;

    if size == 0 { return Ok(OK); }

    unimplemented!()
}

// TODO:
fn may_asynsend_to(_src: Endpoint, _dst: Endpoint, _process_table: &mut MinixProcessTable) -> Result<bool, nix::Error> {
    Ok(true)
}

fn read_asynmsg(addr: u64, process: &MinixProcess) -> Result<AsynMsg, nix::Error> {
    let buf = process.read_buf_u8(addr, size_of::<AsynMsg>())?;

    let mut result = [0u8; size_of::<AsynMsg>()];

    for (dst, src) in result.iter_mut().zip(buf) {
        *dst = src;
    }

    Ok( unsafe { std::mem::transmute(result) } )
}

fn write_asynmsg(asynmsg: AsynMsg, addr: u64, process: &MinixProcess) -> Result<(), nix::Error> {
    let data: [u8; size_of::<AsynMsg>()] = unsafe { std::mem::transmute(asynmsg) };
    process.write_buf_u8(addr, &data)
}