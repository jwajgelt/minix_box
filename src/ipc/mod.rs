use crate::utils::Endpoint;
use crate::utils::{MinixProcessTable, ProcessState};

mod ipcconst {
    pub const SEND: u64 = 1;
    pub const RECEIVE: u64 = 2;
    pub const SENDREC: u64 = 3;
}

pub fn do_ipc(
    caller_endpoint: Endpoint,
    process_table: &mut MinixProcessTable,
) -> Result<(), nix::Error> {
    let process = process_table.get_mut(caller_endpoint).unwrap();
    let mut regs = process.get_regs()?;
    // advance the instruction pointer to the next instruction
    regs.rip += 2;
    process.set_regs(regs).unwrap();

    // the ecx register contains the type of ipc call
    // TODO: export match arms to their own functions
    match regs.rcx {
        ipcconst::SEND => {
            // dest-src in eax register
            let receive_endpoint = regs.rax as Endpoint;
            let message = process.read_message().unwrap(); // TODO: if error here, cause error in process
            match process_table.get_mut(receive_endpoint) {
                Some(receiver) => {
                    if let ProcessState::Receiving(sender) = receiver.state {
                        // TODO: or ANY
                        if sender == caller_endpoint {
                            // write the message to the receiver's memory
                            // and resume it
                            receiver.write_message(message).unwrap(); // TODO: error handling everywhere
                            receiver.state = ProcessState::Running;
                            receiver.cont().unwrap();

                            // resume the sender
                            let sender = process_table.get(caller_endpoint).unwrap(); // unwrap here is safe, since we checked caller_endpoint is in table at the beginning
                            sender.cont().unwrap();
                            return Ok(());
                        }
                        // else - receiver is waiting for a message from another process
                    }
                    // set the sender's state as SENDING to the receiver
                    let sender = process_table.get_mut(caller_endpoint).unwrap(); // unwrap here is safe, like above
                    sender.state = ProcessState::Sending(receive_endpoint);
                    // we don't resume the sender, since it's supposed to be blocked on send
                    Ok(())
                }
                None => {
                    // bad endpoint - minix returns EDEADSRCDST here
                    panic!()
                }
            }
        }
        // TODO: this is, so far, very naive - we don't expect ANY as an endpoint, we don't handle
        // NOTIFYs, async sends, we don't check flags for non-blocking, we don't do message queues,
        // we don't set the return value
        ipcconst::RECEIVE => {
            // dest-src in eax register
            let send_endpoint = regs.rax as Endpoint;

            // TODO: handle the case of send_endpoint being ANY

            match process_table.get_mut(send_endpoint) {
                Some(sender) => {
                    if let ProcessState::Sending(receiver) = sender.state {
                        if receiver == caller_endpoint {
                            // read the message from the sender's memory,
                            // write to to receiver's memory and resume both processes
                            let message = sender.read_message().unwrap(); // TODO: error handling
                            sender.state = ProcessState::Running;
                            sender.cont().unwrap();

                            let receiver = process_table.get(caller_endpoint).unwrap(); // again, unwrap here is safe
                            receiver.write_message(message).unwrap();
                            receiver.cont().unwrap();

                            // TODO: before returning, we should set the return value in
                            // receiver as well - it should be in the ebx register
                            return Ok(());
                        }
                    }
                    // set the receiver's state as RECEIVING from the sender
                    let receiver = process_table.get_mut(caller_endpoint).unwrap(); // unwrap here is safe, again
                    receiver.state = ProcessState::Receiving(send_endpoint);
                    // don't resume the receiver here, since it's supposed to be blocked on receive
                    Ok(())
                }
                None => {
                    // bad endpoint - minix returns EDEADSRCDST here
                    panic!()
                }
            }
        }
        ipcconst::SENDREC => {
            todo!()
        }
        _ => {
            // TODO: handle everything
            todo!()
        }
    }
}
