# Minix Box

Minix Box is a proof-of-concept Minix 3 compatibility layer for Linux.

The goal of the project is to run programs compiled for Minix 3.4 by simulating the Minix system call interface.

## Building the project

Minix Box uses Rust's `cargo` build system, available as part of the [`rustup` toolchain](https://rustup.rs/).

To build the program use `cargo build` in the project's root directory.

To run the program use `cargo run`.

## Current functionality

Currently, the majority of system calls necessary for executing Minix programs has not been implemented yet.

So far, only the IPC calls have been fully implemented.

When the program is ran using `cargo run`, the Minix Box executable starts the Minix Reincarnation Server (RS) and executes it until an unimplemented kernel call is encountered.

## Tests

There are a few tests designed to check some basic functionality of the project.

Tests in `main.rs`:
- `send_receive_test` - spawns two Minix processes, one which sends a message, and one which receives it
- `sendrec_test` - spawns two Minix processes which exchange messages using `sendrec`, `receive` and `send`.

Tests in `utils/minix_process.rs`:
- `do_syscall_test` - spawns a process and injects a `write` Linux system call into it
- `attach_shared_test` - spawns a process and maps shared memory in its address space

Because the `cargo test` implementation uses threads to execute multiple tests at the same time and the current implementation hasn't been designed with such uses in mind, the tests have to be executed one at a time using the command:
```
RUST_MIN_STACK=8388608 cargo test {name_of_test}
```
The `RUST_MIN_STACK` variable is necessary to avoid stack overflows resulting from small default stack size for threads.